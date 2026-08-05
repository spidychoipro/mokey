mod app;
mod controller;
mod keys;

fn main() -> eframe::Result {
    let config = mokey_core::Config::load().unwrap_or_else(|e| {
        eprintln!("config load failed ({e}); using defaults");
        mokey_core::Config::default()
    });
    let mouse = mokey_backend::mouse::create().expect("failed to create mouse backend");
    let input = mokey_backend::hotkey::spawn(
        &config.general.trigger_hotkey,
        &config.general.settings_hotkey,
    )
    .expect("failed to register hotkeys");
    let capture = input.capture.clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_visible(false)
            .with_position([-32000.0, -32000.0])
            .with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "mokey",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(app::MokeyApp::new(
                config,
                mouse,
                input.hotkeys,
                input.keys,
                capture,
            )))
        }),
    )
}
