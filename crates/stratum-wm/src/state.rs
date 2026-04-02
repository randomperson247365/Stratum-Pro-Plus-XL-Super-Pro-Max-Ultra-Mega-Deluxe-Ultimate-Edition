use std::collections::HashMap;

use wayland_client::{
    protocol::{
        wl_buffer, wl_compositor, wl_output, wl_registry, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
    Connection, Dispatch, QueueHandle,
};

use stratum_config::StratumConfig;
use stratum_ipc::IpcMessage;
use tokio::sync::broadcast;

use crate::{
    decorations::{self, TitlebarRenderer, WindowDecoration},
    decorations::renderer::parse_hex_to_rgb,
    keybinds::{execute_action, parse_keybind, ActionContext},
    output::OutputState,
    protocol::{
        river_window_management_v1::{
            river_decoration_v1, river_node_v1, river_output_v1, river_seat_v1,
            river_window_manager_v1, river_window_v1,
        },
        river_xkb_bindings_v1::{river_xkb_binding_v1, river_xkb_bindings_v1},
        river_layer_shell_v1::river_layer_shell_v1,
        river_input_management_v1::river_input_manager_v1,
        river_libinput_config_v1::river_libinput_config_v1,
        RiverDecorationV1, RiverInputManagerV1, RiverLayerShellV1, RiverLibinputConfigV1,
        RiverNodeV1, RiverOutputV1, RiverSeatV1, RiverWindowManagerV1,
        RiverWindowV1, RiverXkbBindingsV1, RiverXkbBindingV1,
    },
    seat::{RegisteredKeybind, SeatState},
    window::WindowState,
};

// ── Globals ──────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Globals {
    pub compositor:      Option<wl_compositor::WlCompositor>,
    pub wl_shm:          Option<wl_shm::WlShm>,
    pub rwm:             Option<RiverWindowManagerV1>,
    pub xkb_bindings:    Option<RiverXkbBindingsV1>,
    pub layer_shell:     Option<RiverLayerShellV1>,
    pub input_manager:   Option<RiverInputManagerV1>,
    pub libinput_config: Option<RiverLibinputConfigV1>,
    pub wl_seats:        HashMap<u32, wl_seat::WlSeat>,
    pub wl_outputs:      HashMap<u32, wl_output::WlOutput>,
}

// ── AppState ─────────────────────────────────────────────────────────────────

pub struct AppState {
    pub globals:           Globals,
    pub windows:           HashMap<u64, WindowState>,
    pub outputs:           HashMap<u64, OutputState>,
    pub seats:             HashMap<u64, SeatState>,
    pub focused_window:    Option<u64>,
    pub focus_stack:       Vec<u64>,
    /// Action deferred until the next manage_start.
    pub pending_action:    Option<String>,
    pub layout_dirty:      bool,
    pub config:            StratumConfig,
    pub running:           bool,
    // Phase 2 — decorations
    pub decorations:       HashMap<u64, WindowDecoration>,
    /// Maps titlebar wl_surface protocol IDs back to their window.
    pub surface_to_window: HashMap<u32, u64>,
    pub font_renderer:     TitlebarRenderer,
    // Phase 3 — IPC
    pub ipc_tx:            Option<broadcast::Sender<IpcMessage>>,
}

impl AppState {
    pub fn new(config: StratumConfig) -> Self {
        Self {
            globals:           Globals::default(),
            windows:           HashMap::new(),
            outputs:           HashMap::new(),
            seats:             HashMap::new(),
            focused_window:    None,
            focus_stack:       Vec::new(),
            pending_action:    None,
            layout_dirty:      false,
            config,
            running:           true,
            decorations:       HashMap::new(),
            surface_to_window: HashMap::new(),
            font_renderer:     TitlebarRenderer::new(),
            ipc_tx:            None,
        }
    }

    pub fn set_ipc_tx(&mut self, tx: broadcast::Sender<IpcMessage>) {
        self.ipc_tx = Some(tx);
    }

    /// Cheap identifier for a Wayland proxy object.
    fn obj_id(proxy: &impl wayland_client::Proxy) -> u64 {
        proxy.id().protocol_id() as u64
    }

    pub fn register_keybinds(&mut self, qh: &QueueHandle<Self>) {
        let xkb = match self.globals.xkb_bindings.clone() {
            Some(b) => b,
            None => {
                eprintln!("stratum-wm: river_xkb_bindings_v1 not available — keybinds disabled");
                return;
            }
        };

        let binds: Vec<(String, String)> = self.config.keybindings.0.clone().into_iter().collect();
        let seat_ids: Vec<u64> = self.seats.keys().copied().collect();

        for seat_id in seat_ids {
            if let Some(seat) = self.seats.get_mut(&seat_id) {
                seat.clear_keybinds();
            }
            let river_seat = match self.seats.get(&seat_id).map(|s| s.proxy.clone()) {
                Some(s) => s,
                None => continue,
            };
            for (spec, action) in &binds {
                if let Some(kb) = parse_keybind(spec, action) {
                    // Convert u32 keysym/modifiers to the generated bitflags types.
                    let keysym = kb.keysym.raw();
                    let mods = river_seat_v1::Modifiers::from_bits_truncate(kb.modifiers);
                    let binding =
                        xkb.get_xkb_binding(&river_seat, keysym, mods, qh, action.clone());
                    binding.enable();
                    if let Some(seat) = self.seats.get_mut(&seat_id) {
                        seat.registered_binds.push(RegisteredKeybind {
                            binding,
                            action: action.clone(),
                        });
                    }
                }
            }
        }
    }

    fn handle_manage_start(&mut self, qh: &QueueHandle<Self>) {
        if let Some(action) = self.pending_action.take() {
            let terminal = self.config.general.terminal.clone();
            let focused_proxy = self
                .focused_window
                .and_then(|id| self.windows.get(&id))
                .map(|w| w.proxy.clone());

            match action.as_str() {
                "focus_next" => self.focus_next(),
                "toggle_fullscreen" => {
                    // fullscreen request requires an output; use the first known output.
                    let output = self.outputs.values().next().map(|o| o.proxy.clone());
                    if let (Some(win_proxy), Some(out)) = (focused_proxy, output) {
                        win_proxy.fullscreen(&out);
                    }
                }
                _ => {
                    execute_action(
                        &action,
                        ActionContext {
                            terminal: &terminal,
                            focused_window: focused_proxy.as_ref(),
                        },
                    );
                }
            }
        }

        if self.layout_dirty {
            self.layout_dirty = false;
            self.apply_floating_layout_manage();
        }

        // CRITICAL: every manage_start MUST be followed by manage_finish.
        if let Some(rwm) = self.globals.rwm.clone() {
            rwm.manage_finish();
        }
    }

    fn apply_floating_layout_manage(&self) {
        let (ow, oh) = self
            .outputs
            .values()
            .next()
            .map(|o| (o.width as i32, o.height as i32))
            .unwrap_or((1920, 1080));

        for win in self.windows.values() {
            if win.minimized {
                win.proxy.hide();
                continue;
            }
            win.proxy.show();
            let w = win.width.max(400).min(ow);
            let h = win.height.max(300).min(oh);
            win.proxy.propose_dimensions(w, h);
            win.proxy.use_ssd();
        }
    }

    fn apply_floating_layout_render(&mut self, qh: &QueueHandle<Self>) {
        let (ow, oh) = self
            .outputs
            .values()
            .next()
            .map(|o| (o.width as i32, o.height as i32))
            .unwrap_or((1920, 1080));

        // Snapshot per-window data to avoid holding a borrow on self.windows
        // while also mutably borrowing self.decorations later in the loop.
        let window_data: Vec<(u64, i32, i32, i32, i32, bool, bool, bool, String)> = self
            .windows
            .iter()
            .map(|(id, win)| {
                let is_active = self.focused_window == Some(*id);
                let actual_w = if win.actual_width > 0 { win.actual_width } else { win.width };
                let title = win.display_title().to_owned();
                (*id, win.x, win.y, win.width, actual_w, win.minimized, win.fullscreen, is_active, title)
            })
            .collect();

        use crate::protocol::river_window_management_v1::river_window_v1::Edges;

        for (win_id, win_x, win_y, win_w, actual_w, minimized, fullscreen, is_active, title) in window_data {
            if minimized {
                continue;
            }

            // Position the window node.
            if let Some(win) = self.windows.get(&win_id) {
                let node = win.proxy.get_node(qh, ());
                let x = if win_x == 0 { (ow - win_w).max(0) / 2 } else { win_x };
                let y = if win_y == 0 { (oh - win_w).max(0) / 2 } else { win_y };
                node.set_position(x, y);

                // Protocol borders (compositor-drawn; free).
                if !fullscreen {
                    let (bw, br, bg, bb) = if is_active {
                        let (r, g, b) = parse_hex_to_rgb(&self.config.appearance.accent_color);
                        (self.config.decorations.border_width_active as i32, r, g, b)
                    } else {
                        (self.config.decorations.border_width_inactive as i32, 0x55u32, 0x55u32, 0x55u32)
                    };
                    let edges = Edges::Top | Edges::Bottom | Edges::Left | Edges::Right;
                    win.proxy.set_borders(edges, bw, br, bg, bb, 0xff);
                } else {
                    win.proxy.set_borders(Edges::empty(), 0, 0, 0, 0, 0);
                }
            }

            // Update and commit the CPU-rendered titlebar surface.
            if !fullscreen {
                if let Some(wl_shm) = self.globals.wl_shm.clone() {
                    if let Some(deco) = self.decorations.get_mut(&win_id) {
                        decorations::update(
                            deco, &wl_shm, qh, actual_w,
                            is_active, &title, &self.config, &self.font_renderer,
                        );
                        decorations::commit_in_render_sequence(deco);
                    }
                }
            }
        }
    }

    pub fn focus_next(&mut self) {
        if self.focus_stack.len() < 2 {
            return;
        }
        let current = self.focused_window;
        let next = self
            .focus_stack
            .iter()
            .position(|&id| Some(id) == current)
            .map(|pos| self.focus_stack[(pos + 1) % self.focus_stack.len()])
            .or_else(|| self.focus_stack.first().copied());
        if let Some(next_id) = next {
            self.set_focus(next_id);
        }
    }

    pub fn set_focus(&mut self, window_id: u64) {
        let seat_proxy = self.seats.values().next().map(|s| s.proxy.clone());
        if let (Some(seat), Some(win)) = (seat_proxy, self.windows.get(&window_id)) {
            seat.focus_window(&win.proxy);
        }
        self.focused_window = Some(window_id);
        self.focus_stack.retain(|&id| id != window_id);
        self.focus_stack.insert(0, window_id);

        // Broadcast focus change over IPC.
        if let Some(tx) = &self.ipc_tx {
            if let Some(win) = self.windows.get(&window_id) {
                let _ = tx.send(IpcMessage::FocusChanged {
                    app_id: win.app_id.clone().unwrap_or_default(),
                    title:  win.display_title().to_owned(),
                });
            }
        }
    }

    pub fn remove_window(&mut self, window_id: u64) {
        // Clean up decoration surface before dropping the window.
        if let Some(deco) = self.decorations.remove(&window_id) {
            self.surface_to_window
                .remove(&(Self::obj_id(&deco.titlebar_surface) as u32));
            decorations::destroy(deco);
        }
        self.windows.remove(&window_id);
        self.focus_stack.retain(|&id| id != window_id);
        if self.focused_window == Some(window_id) {
            self.focused_window = self.focus_stack.first().copied();
            if let Some(next_id) = self.focused_window {
                self.set_focus(next_id);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Dispatch implementations
// ═══════════════════════════════════════════════════════════════════════════

impl Dispatch<wl_registry::WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "river_window_manager_v1" => {
                    state.globals.rwm =
                        Some(registry.bind(name, version.min(3), qh, ()));
                }
                "river_xkb_bindings_v1" => {
                    state.globals.xkb_bindings =
                        Some(registry.bind(name, version.min(1), qh, ()));
                }
                "river_layer_shell_v1" => {
                    state.globals.layer_shell =
                        Some(registry.bind(name, version.min(1), qh, ()));
                }
                "river_input_manager_v1" => {
                    state.globals.input_manager =
                        Some(registry.bind(name, version.min(1), qh, ()));
                }
                "river_libinput_config_v1" => {
                    state.globals.libinput_config =
                        Some(registry.bind(name, version.min(1), qh, ()));
                }
                "wl_compositor" => {
                    state.globals.compositor =
                        Some(registry.bind(name, version.min(5), qh, ()));
                }
                "wl_shm" => {
                    state.globals.wl_shm =
                        Some(registry.bind(name, version.min(1), qh, ()));
                }
                "wl_seat" => {
                    let seat: wl_seat::WlSeat = registry.bind(name, version.min(7), qh, name);
                    state.globals.wl_seats.insert(name, seat);
                }
                "wl_output" => {
                    let output: wl_output::WlOutput =
                        registry.bind(name, version.min(4), qh, name);
                    state.globals.wl_outputs.insert(name, output);
                }
                _ => {}
            }
        }
    }
}

// ── RiverWindowManagerV1 ────────────────────────────────────────────────────

impl Dispatch<RiverWindowManagerV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &RiverWindowManagerV1,
        event: river_window_manager_v1::Event,
        _: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            river_window_manager_v1::Event::Unavailable => {
                eprintln!(
                    "stratum-wm: river_window_manager_v1 unavailable \
                     (another window manager is running?)"
                );
                state.running = false;
            }
            river_window_manager_v1::Event::ManageStart => {
                state.handle_manage_start(qh);
            }
            river_window_manager_v1::Event::RenderStart => {
                state.apply_floating_layout_render(qh);
                if let Some(rwm) = state.globals.rwm.clone() {
                    rwm.render_finish();
                }
            }
            river_window_manager_v1::Event::Window { id } => {
                let win_id = Self::obj_id(&id);
                state.windows.insert(win_id, WindowState::new(id.clone()));
                state.focus_stack.push(win_id);
                state.layout_dirty = true;

                // Create a titlebar decoration surface for this window.
                if let (Some(comp), Some(shm)) = (
                    state.globals.compositor.clone(),
                    state.globals.wl_shm.clone(),
                ) {
                    match decorations::create(&id, &comp, &shm, qh, &state.config) {
                        Ok(deco) => {
                            let surf_id = Self::obj_id(&deco.titlebar_surface) as u32;
                            state.surface_to_window.insert(surf_id, win_id);
                            state.decorations.insert(win_id, deco);
                        }
                        Err(e) => eprintln!("stratum-wm: decoration create failed: {e}"),
                    }
                }
            }
            river_window_manager_v1::Event::Output { id } => {
                let out_id = Self::obj_id(&id);
                state.outputs.insert(out_id, OutputState::new(id));
            }
            river_window_manager_v1::Event::Seat { id } => {
                let seat_id = Self::obj_id(&id);
                state.seats.insert(seat_id, SeatState::new(id));
                state.register_keybinds(qh);
            }
            river_window_manager_v1::Event::SessionLocked => {}
            river_window_manager_v1::Event::SessionUnlocked => {}
            river_window_manager_v1::Event::Finished => {
                state.running = false;
            }
            _ => {}
        }
    }
}

// ── RiverWindowV1 ────────────────────────────────────────────────────────────

impl Dispatch<RiverWindowV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &RiverWindowV1,
        event: river_window_v1::Event,
        _: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let win_id = Self::obj_id(proxy);

        match event {
            river_window_v1::Event::Closed => {
                state.remove_window(win_id);
                proxy.destroy();
            }
            river_window_v1::Event::AppId { app_id } => {
                if let Some(win) = state.windows.get_mut(&win_id) {
                    win.app_id = app_id; // already Option<String>
                }
            }
            river_window_v1::Event::Title { title } => {
                if let Some(win) = state.windows.get_mut(&win_id) {
                    win.title = title; // already Option<String>
                }
            }
            river_window_v1::Event::Dimensions { width, height } => {
                if let Some(win) = state.windows.get_mut(&win_id) {
                    win.actual_width  = width;
                    win.actual_height = height;
                }
            }
            river_window_v1::Event::FullscreenRequested { .. } => {
                if let Some(win) = state.windows.get_mut(&win_id) {
                    win.fullscreen = true;
                }
                proxy.inform_fullscreen();
                state.layout_dirty = true;
            }
            river_window_v1::Event::ExitFullscreenRequested => {
                if let Some(win) = state.windows.get_mut(&win_id) {
                    win.fullscreen = false;
                }
                proxy.inform_not_fullscreen();
                state.layout_dirty = true;
            }
            river_window_v1::Event::MaximizeRequested => {
                proxy.inform_maximized();
                state.layout_dirty = true;
            }
            river_window_v1::Event::UnmaximizeRequested => {
                proxy.inform_unmaximized();
                state.layout_dirty = true;
            }
            river_window_v1::Event::MinimizeRequested => {
                if let Some(win) = state.windows.get_mut(&win_id) {
                    win.minimized = true;
                }
                state.layout_dirty = true;
            }
            river_window_v1::Event::PointerMoveRequested { .. }
            | river_window_v1::Event::PointerResizeRequested { .. } => {
                // Notify River the operation is starting; full drag is Phase 2.
                proxy.inform_resize_start();
            }
            river_window_v1::Event::DecorationHint { .. }
            | river_window_v1::Event::DimensionsHint { .. }
            | river_window_v1::Event::Parent { .. }
            | river_window_v1::Event::UnreliablePid { .. }
            | river_window_v1::Event::ShowWindowMenuRequested { .. } => {}
            _ => {}
        }
    }
}

// ── RiverOutputV1 ────────────────────────────────────────────────────────────

impl Dispatch<RiverOutputV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &RiverOutputV1,
        event: river_output_v1::Event,
        _: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let out_id = Self::obj_id(proxy);
        match event {
            river_output_v1::Event::Dimensions { width, height } => {
                if let Some(out) = state.outputs.get_mut(&out_id) {
                    out.width  = width as u32;
                    out.height = height as u32;
                }
            }
            river_output_v1::Event::Position { x, y } => {
                if let Some(out) = state.outputs.get_mut(&out_id) {
                    out.x = x;
                    out.y = y;
                }
            }
            river_output_v1::Event::WlOutput { name } => {
                if let Some(out) = state.outputs.get_mut(&out_id) {
                    out.wl_output_name = Some(name);
                }
            }
            river_output_v1::Event::Removed => {
                state.outputs.remove(&out_id);
                proxy.destroy();
            }
            _ => {}
        }
    }
}

// ── RiverSeatV1 ─────────────────────────────────────────────────────────────

impl Dispatch<RiverSeatV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &RiverSeatV1,
        event: river_seat_v1::Event,
        _: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let seat_id = Self::obj_id(proxy);
        match event {
            river_seat_v1::Event::WlSeat { name } => {
                if let Some(seat) = state.seats.get_mut(&seat_id) {
                    seat.wl_seat_name = Some(name);
                }
            }
            river_seat_v1::Event::Removed => {
                if let Some(mut seat) = state.seats.remove(&seat_id) {
                    seat.clear_keybinds();
                }
                proxy.destroy();
            }
            river_seat_v1::Event::WindowInteraction { window } => {
                let win_id = Self::obj_id(&window);
                if state.windows.contains_key(&win_id) {
                    state.set_focus(win_id);
                }
            }
            _ => {}
        }
    }
}

// ── RiverXkbBindingsV1 ───────────────────────────────────────────────────────

impl Dispatch<RiverXkbBindingsV1, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &RiverXkbBindingsV1,
        _: river_xkb_bindings_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

// ── RiverXkbBindingV1 ────────────────────────────────────────────────────────

impl Dispatch<RiverXkbBindingV1, String> for AppState {
    fn event(
        state: &mut Self,
        _proxy: &RiverXkbBindingV1,
        event: river_xkb_binding_v1::Event,
        action: &String,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            river_xkb_binding_v1::Event::Pressed => {
                // Store the action; River will fire manage_start next.
                state.pending_action = Some(action.clone());
            }
            river_xkb_binding_v1::Event::Released => {
                // manage_start follows; nothing extra on release.
            }
            _ => {}
        }
    }
}

// ── RiverDecorationV1 ────────────────────────────────────────────────────────

impl Dispatch<RiverDecorationV1, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &RiverDecorationV1,
        _: river_decoration_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

// ── RiverNodeV1 ──────────────────────────────────────────────────────────────

impl Dispatch<RiverNodeV1, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &RiverNodeV1,
        _: river_node_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

// ── RiverLayerShellV1 / RiverInputManagerV1 / RiverLibinputConfigV1 ──────────

impl Dispatch<RiverLayerShellV1, ()> for AppState {
    fn event(
        _: &mut Self, _: &RiverLayerShellV1,
        _: river_layer_shell_v1::Event,
        _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<RiverInputManagerV1, ()> for AppState {
    fn event(
        _: &mut Self, _: &RiverInputManagerV1,
        _: river_input_manager_v1::Event,
        _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<RiverLibinputConfigV1, ()> for AppState {
    fn event(
        _: &mut Self, _: &RiverLibinputConfigV1,
        _: river_libinput_config_v1::Event,
        _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

// ── Standard Wayland stubs ────────────────────────────────────────────────────

impl Dispatch<wl_compositor::WlCompositor, ()> for AppState {
    fn event(
        _: &mut Self, _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event, _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppState {
    fn event(
        _: &mut Self, _: &wl_shm::WlShm,
        _: wl_shm::Event, _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppState {
    fn event(
        _: &mut Self, _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event, _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for AppState {
    fn event(
        _: &mut Self, _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event, _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppState {
    fn event(
        _: &mut Self, _: &wl_surface::WlSurface,
        _: wl_surface::Event, _: &(), _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, u32> for AppState {
    fn event(
        _: &mut Self, _: &wl_seat::WlSeat,
        _: wl_seat::Event, _: &u32, _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, u32> for AppState {
    fn event(
        _: &mut Self, _: &wl_output::WlOutput,
        _: wl_output::Event, _: &u32, _: &Connection, _: &QueueHandle<Self>,
    ) {
    }
}
