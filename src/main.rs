#![cfg_attr(windows, windows_subsystem = "windows")]
mod cli_app;
mod config;
mod constants;
mod gui_app;
mod network;
mod pad_state;
mod poller_worker;
use cli_app::CliApp;
use eframe::{NativeOptions, egui};
use gui_app::GuiApp;

fn main() -> eframe::Result<()> {
    // If the app is runned with --gui flag, run the GUI app
    Ok(if std::env::args().any(|arg| arg == "--cli") {
        let _ = CliApp::new().run();
    } else {
        const ICON_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app_icon.rgba"));

        let width = u32::from_le_bytes([ICON_DATA[0], ICON_DATA[1], ICON_DATA[2], ICON_DATA[3]]);
        let height = u32::from_le_bytes([ICON_DATA[4], ICON_DATA[5], ICON_DATA[6], ICON_DATA[7]]);
        let rgba = ICON_DATA[8..].to_vec();

        let icon = egui::IconData {
            width,
            height,
            rgba,
        };

        let options = NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([350.0, 400.0])
                .with_resizable(false)
                .with_icon(icon),
            ..Default::default()
        };

        let _ = eframe::run_native(
            "3DS Input Redirection",
            options,
            Box::new(|_cc| Ok(Box::new(GuiApp::new()) as Box<dyn eframe::App>)),
        );
    })
}
