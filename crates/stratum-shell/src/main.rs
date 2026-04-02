mod app;
mod ipc;
mod panel;

fn main() -> anyhow::Result<()> {
    app::run()
}
