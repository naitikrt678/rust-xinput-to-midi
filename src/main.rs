// #![windows_subsystem = "windows"]  // commented out for debug — re-enable for release

mod app;
mod config;
mod input;
mod midi;
mod parser;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Controller → MIDI")
            .with_inner_size([680.0, 560.0])
            .with_min_inner_size([560.0, 440.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Controller → MIDI",
        native_options,
        Box::new(|cc| Box::new(app::ControllerMidiApp::new(cc))),
    )
}
