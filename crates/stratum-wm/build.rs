fn main() {
    for name in &[
        "river-window-management-v1",
        "river-xkb-bindings-v1",
        "river-layer-shell-v1",
        "river-input-management-v1",
        "river-libinput-config-v1",
    ] {
        println!("cargo:rerun-if-changed=protocol/{name}.xml");
    }
}
