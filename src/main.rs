#![cfg_attr(windows, windows_subsystem = "windows")]

mod config;
mod constants;
mod gui_app;
mod network;
mod pad_state;
use eframe::{NativeOptions, egui};
use gui_app::GuiApp;

fn main() -> eframe::Result<()> {
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

    eframe::run_native(
        "3DS Input Redirection",
        options,
        Box::new(|cc| Ok(Box::new(GuiApp::new(cc)))),
    )
}
