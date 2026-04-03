# Stratum Pro Plus XL Super Pro Max Ultra Mega Deluxe Ultimate Edition

# I have no idea if this even works this is just to show that if an AI can build a working desktop environment with this anyone can (it is still untested so I seriously have no idea if it works AI can make very stupid mistakes and does quite often so be careful) 

A desktop environment built in Rust on the [River](https://codeberg.org/river/river) Wayland compositor. River handles all rendering; Stratum handles window management policy, decorations, the panel, launcher, and settings.

## Phase Status

- [x] Phase 1 — Foundation (workspace, River WM protocol, floating layout, keybinds)
- [x] Phase 2 — Decorations (titlebars, borders, shadows)
- [x] Phase 3 — Shell & Panel (stratum-shell, bottom panel, clock/battery/network/window-title widgets, IPC)
- [x] Phase 4 — App Launcher (XDG .desktop scanner, fuzzy search, full-screen overlay via Super+Space)
- [x] Phase 5 — Tiling Mode (master-stack + BSP layouts, Super+T cycle, deck auto-fallback, DPI-aware min-tile thresholds)
- [x] Phase 6 — Settings App (Iced GUI: Appearance / Decorations / Keybindings tabs, TOML save)
- [x] Phase 7 — Polish & Ship (window slide-in animations, AUR PKGBUILD, Nix flake)

---

## Dependencies

| Package | Install (Arch/CachyOS) | Purpose |
|---------|------------------------|---------|
| `river` | `paru -S river` | Wayland compositor |
| `foot` | `paru -S foot` | Default terminal |
| `libxkbcommon` | `sudo pacman -S libxkbcommon` | Keyboard support |
| `rust >= 1.77` | `rustup update` | Build toolchain |

---

## Build

```bash
cargo build --release
```

---

## Install (Arch / CachyOS)

Run these after building:

```bash
sudo install -Dm755 target/release/stratum-wm        /usr/local/bin/stratum-wm
sudo install -Dm755 target/release/stratum-shell     /usr/local/bin/stratum-shell
sudo install -Dm755 target/release/stratum-settings  /usr/local/bin/stratum-settings
sudo install -Dm644 data/stratum-settings.desktop    /usr/share/applications/stratum-settings.desktop
sudo install -Dm755 contrib/river-init               /usr/local/bin/stratum-river-init
sudo install -Dm644 data/stratum.desktop             /usr/share/wayland-sessions/stratum.desktop
sudo install -Dm644 data/default-config.toml         /etc/stratum/config.toml
```

Then **log out of KDE** and select **Stratum** from the SDDM session picker.

SDDM reads session files from `/usr/share/wayland-sessions/` — Stratum will appear
alongside "KDE Plasma (Wayland)" without any further configuration.

---

## How It Works

```
SDDM → launches River (via stratum.desktop)
         └─ River runs contrib/river-init
                 └─ river-init launches stratum-wm
                         └─ stratum-wm connects to River via
                            river-window-management-v1 protocol
                            and handles all WM policy
```

River handles Wayland rendering. `stratum-wm` is the policy layer: it manages
window positions, draws server-side decorations (titlebars, borders, shadows),
dispatches keybindings, and communicates with the shell via IPC.

`stratum-shell` runs as a separate process, anchored to the bottom of the screen
as a Wayland layer-shell surface. It receives focus/workspace events over the IPC
socket and expands to a full-screen launcher overlay on `Super+Space`.

---

## Config

| Location | Purpose |
|----------|---------|
| `~/.config/stratum/config.toml` | Per-user config (takes priority) |
| `/etc/stratum/config.toml` | System-wide defaults |
| Built-in defaults | Fallback if no file found |

Config is **hot-reloaded on save** via inotify — no restart needed.

### Default Keybindings

| Key | Action |
|-----|--------|
| `Super+Return` | Open terminal (foot) |
| `Super+Space` | Open app launcher |
| `Super+Q` | Close focused window |
| `Super+F` | Toggle fullscreen |
| `Super+T` | Cycle layout mode (Floating → Master-Stack → BSP → …) |
| `Super+Tab` | Focus next window |
| `Super+Ctrl+F1..F5` | Switch VT |

All bindings are configurable in `config.toml`:

```toml
[keybindings]
"super+Return" = "spawn_terminal"
"super+q"      = "close_focused"
"super+f"      = "toggle_fullscreen"
"super+Tab"    = "focus_next"
```

---

## Layout Modes

Press `Super+T` to cycle through layout modes. The switch is per-session and hot —
windows rearrange immediately.

| Mode | `Super+T` order | Description |
|------|----------------|-------------|
| Floating | 1st | Free-form windows with titlebars |
| Master-Stack | 2nd | Left master pane + evenly-split right stack |
| BSP | 3rd | Recursive binary-space partition (alternates vertical/horizontal splits) |

In tiling modes (Master-Stack and BSP) titlebars are hidden; compositor-drawn borders remain active.

### Deck auto-fallback

When the output is too small to fit all tiled windows above the minimum readable
size, both tiling modes automatically switch to a *deck* arrangement: all windows
are stacked at the **focused window's computed tile position** rather than rendered
as tiny, unusable panes. Cycling focus with `Super+Tab` shifts the deck to the next
window's tile position, so the spatial layout stays intact — just stacked instead of
side-by-side.

The minimum tile size is configured in pixels at 96 dpi. When the display reports
its physical dimensions (`wl_output::Geometry`), the thresholds are scaled
proportionally so they represent the same physical area on any density screen.

```toml
[layout]
default_mode    = "floating"   # floating | master_stack | bsp
min_tile_width  = 400          # px at 96 dpi
min_tile_height = 280          # px at 96 dpi
```

Gap sizes apply to all tiling modes:

```toml
[appearance]
gap_outer = 12   # pixels between windows and screen edge
gap_inner = 8    # pixels between windows
```

---

## App Launcher

Press `Super+Space` to open the launcher overlay. stratum-shell expands to fill
the screen, showing a fuzzy-search box and a list of all XDG `.desktop` applications
found in `/usr/share/applications` and `~/.local/share/applications`.

- Type to filter (case-insensitive, ranked by prefix > contains)
- Click a row or press `Enter` to launch
- Press `Escape` to dismiss

```toml
[launcher]
show_recently_used = true
show_categories    = true
max_recent         = 8
search_settings    = true
```

---

## Decorations Config

```toml
[decorations]
titlebar_height     = 28
border_width_active = 2
border_width_inactive = 1
border_radius       = 8
shadow_enabled      = true
shadow_spread       = 12
shadow_opacity      = 0.4
buttons_position    = "right"
buttons             = ["minimize", "maximize", "close"]
```

---

## Crate Structure

| Crate | Binary | Purpose |
|-------|--------|---------|
| `stratum-wm` | `stratum-wm` | Window manager (River protocol client) |
| `stratum-config` | — | Config schema, TOML load/save, inotify watcher |
| `stratum-ipc` | — | Unix socket IPC (JSON pub/sub) |
| `stratum-shell` | `stratum-shell` | Bottom panel + app launcher overlay |
| `stratum-settings` | `stratum-settings` | GUI settings app *(Phase 6)* |
| `stratum-session` | `stratum-session` | Autostart runner *(Phase 1 stub)* |

---

## Packaging

### Arch Linux / CachyOS (AUR)

```bash
git clone https://github.com/randomperson247365/Stratum-Pro-Plus-XL-Super-Pro-Max-Ultra-Mega-Deluxe-Ultimate-Edition
cd Stratum-Pro-Plus-XL-Super-Pro-Max-Ultra-Mega-Deluxe-Ultimate-Edition/pkg
makepkg -si
```

Or install via an AUR helper once the package is published:
```bash
paru -S stratum-de
```

### Nix / NixOS

```bash
# Run without installing
nix run github:randomperson247365/Stratum-Pro-Plus-XL-Super-Pro-Max-Ultra-Mega-Deluxe-Ultimate-Edition#stratum-wm

# Enter development shell
nix develop github:randomperson247365/Stratum-Pro-Plus-XL-Super-Pro-Max-Ultra-Mega-Deluxe-Ultimate-Edition
```

Add to a NixOS flake:
```nix
inputs.stratum-de.url = "github:randomperson247365/Stratum-Pro-Plus-XL-Super-Pro-Max-Ultra-Mega-Deluxe-Ultimate-Edition";
environment.systemPackages = [ inputs.stratum-de.packages.${system}.default ];
```

---

## Settings App

Launch `stratum-settings` from the app launcher (`Super+Space` → "Stratum Settings")
or run it directly.

It edits `~/.config/stratum/config.toml` directly. stratum-wm picks up the saved
file automatically via inotify hot-reload — no restart needed.

**Tabs:**

| Tab | What you can change |
|-----|---------------------|
| Appearance | Accent colour, dark/light theme, gap sizes, UI/mono fonts |
| Decorations | Titlebar height, border widths, corner radius, shadow |
| Keybindings | Add / remove / remap any `super+key → action` binding |

Click **Save** to write the file; click **Reset** to reload from disk.

---

## Contributing / Hacking

```bash
# Watch for compile errors while editing
cargo watch -x check

# Run just the WM (requires River already running)
cargo run --bin stratum-wm

# Check for warnings
cargo clippy
```
