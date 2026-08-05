mod app;
mod controller;

fn main() -> eframe::Result {
    let config = mokey_core::Config::load().unwrap_or_else(|e| {
        eprintln!("config load failed ({e}); using defaults");
        mokey_core::Config::default()
    });
    let mouse = mokey_backend::mouse::create().expect("failed to create mouse backend");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_visible(true)
            .with_position([-32000.0, -32000.0])
            .with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "mokey",
        options,
        Box::new(move |cc| Ok(Box::new(app::MokeyApp::new(cc, config, mouse)))),
    )
}
