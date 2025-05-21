mod app;
mod config;
mod constants;
mod network;
mod pad_state;

fn main() -> eframe::Result<()> {
    use eframe::{NativeOptions, egui};

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 480.0])
            .with_min_inner_size([350.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust 3DS Input Redirection",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
