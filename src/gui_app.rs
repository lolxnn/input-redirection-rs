// gui_app.rs
use eframe::{App, egui};
use gilrs::Gilrs;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::{
    config::AppConfig,
    poller_worker::{PollerConfig, PollerWorker},
};

pub struct GuiApp {
    // GUI input fields
    target_ip_str: String,
    invert_lx: bool,
    invert_ly: bool,
    invert_rx: bool,
    invert_ry: bool,
    deadzone_lstick_f32: f32,
    deadzone_rstick_f32: f32,

    // PollerWorker management
    poller_running_signal: Option<Arc<AtomicBool>>,
    poller_worker_handle: Option<JoinHandle<()>>,

    // Status message
    status_message: String,
}

impl GuiApp {
    pub fn new() -> Self {
        let app_config = AppConfig::load().unwrap_or_default();

        Self {
            target_ip_str: app_config.target_ip,
            invert_lx: app_config.invert_lx,
            invert_ly: app_config.invert_ly,
            invert_rx: app_config.invert_rx,
            invert_ry: app_config.invert_ry,
            deadzone_lstick_f32: app_config.deadzone_lstick,
            deadzone_rstick_f32: app_config.deadzone_rstick,
            poller_running_signal: None,
            poller_worker_handle: None,
            status_message: "Ready. Configure and start Input-Redirection.".to_string(),
        }
    }

    fn start_poller(&mut self) {
        if self.poller_worker_handle.is_some() {
            self.status_message = "Input-Redirection is already running or finishing.".to_string();
            return;
        }

        // --- SAVE CURRENT GUI STATE TO CONFIG ---
        let new_config = AppConfig {
            target_ip: self.target_ip_str.clone(),
            invert_lx: self.invert_lx,
            invert_ly: self.invert_ly,
            invert_rx: self.invert_rx,
            invert_ry: self.invert_ry,
            deadzone_lstick: self.deadzone_lstick_f32,
            deadzone_rstick: self.deadzone_rstick_f32,
        };

        if let Err(e) = new_config.save() {
            self.status_message = format!("Failed to save config: {:?}", e);
            return;
        }

        // 1. Initialize Gilrs
        let gilrs_instance = match Gilrs::new() {
            Ok(g) => g,
            Err(e) => {
                self.status_message = format!("Failed to initialize Gilrs: {:?}", e);
                return;
            }
        };

        // 2. Find an active gamepad
        let (active_id, gamepad_name) = match gilrs_instance.gamepads().next() {
            Some((id, gamepad)) => (id, gamepad.name().to_string()),
            None => {
                self.status_message = "No gamepad connected. Please connect a gamepad.".to_string();
                return;
            }
        };

        // 3. Create PollerConfig from current GUI state
        let poller_config = PollerConfig {
            target_ip: self.target_ip_str.clone(),
            deadzone_lstick: self.deadzone_lstick_f32,
            deadzone_rstick: self.deadzone_rstick_f32,
            invert_lx: self.invert_lx,
            invert_ly: self.invert_ly,
            invert_rx: self.invert_rx,
            invert_ry: self.invert_ry,
        };

        // 4. Prepare running signal and spawn worker
        let running_signal = Arc::new(AtomicBool::new(true));
        self.poller_running_signal = Some(running_signal.clone());

        let mut poller_worker =
            PollerWorker::new(gilrs_instance, active_id, poller_config, running_signal);

        let handle = thread::spawn(move || {
            poller_worker.run(); // This function now prints to console from the worker
        });

        self.poller_worker_handle = Some(handle);
        self.status_message = format!(
            "Config saved; Input-Redirection started with gamepad: '{}'",
            gamepad_name
        );
    }

    fn stop_poller(&mut self) {
        if let Some(signal) = &self.poller_running_signal {
            signal.store(false, Ordering::SeqCst);
            self.status_message = "Stop signal sent to Input-Redirection. It will stop shortly.".to_string();
            // The update loop will handle joining the thread once it's finished.
        } else {
            self.status_message = "Input-Redirection is not currently running.".to_string();
        }
    }
}

impl App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if the poller thread has finished and handle joining
        let mut poller_just_stopped = false;
        if let Some(handle) = &self.poller_worker_handle {
            if handle.is_finished() {
                poller_just_stopped = true;
            }
        }

        if poller_just_stopped {
            if let Some(handle_to_join) = self.poller_worker_handle.take() {
                match handle_to_join.join() {
                    Ok(_) => {
                        self.status_message =
                            "Input-Redirection thread finished and joined successfully.".to_string();
                    }
                    Err(e) => {
                        self.status_message = format!("Input-Redirection thread panicked: {:?}", e);
                    }
                }
            }
            // Clear the signal as the poller is no longer active
            self.poller_running_signal = None;
        }

        let is_poller_active = self.poller_worker_handle.is_some();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gamepad Input-Redirection Configuration");
            ui.add_space(10.0);

            // --- Configuration Group ---
            // Configuration fields are disabled if the poller is active
            ui.group(|ui| {
                ui.add_enabled_ui(!is_poller_active, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Target IP:");
                        ui.text_edit_singleline(&mut self.target_ip_str);
                    });
                    ui.add_space(5.0);

                    ui.label("Left Stick Deadzone:");
                    ui.add(
                        egui::Slider::new(&mut self.deadzone_lstick_f32, 0.0..=0.99).step_by(0.01),
                    );
                    ui.label("Right Stick Deadzone:");
                    ui.add(
                        egui::Slider::new(&mut self.deadzone_rstick_f32, 0.0..=0.99).step_by(0.01),
                    );
                    ui.add_space(5.0);

                    ui.label("Axis Inversions:");
                    ui.checkbox(&mut self.invert_lx, "Invert Left Stick X");
                    ui.checkbox(&mut self.invert_ly, "Invert Left Stick Y");
                    ui.checkbox(&mut self.invert_rx, "Invert Right Stick X");
                    ui.checkbox(&mut self.invert_ry, "Invert Right Stick Y");
                });
            });

            ui.separator();

            // --- Control Buttons ---
            ui.horizontal(|ui| {
                if is_poller_active {
                    if ui.button("Stop Input-Redirection").clicked() {
                        self.stop_poller();
                    }
                } else {
                    if ui.button("Start Input-Redirection").clicked() {
                        self.start_poller();
                    }
                }
            });

            ui.separator();

            // --- Status Display ---
            ui.label("Status:");
            ui.label(&self.status_message);
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Ensure poller is signaled to stop when GUI exits
        if let Some(signal) = &self.poller_running_signal {
            signal.store(false, Ordering::SeqCst);
        }
        if let Some(handle) = self.poller_worker_handle.take() {
            println!("Attempting to join Input-Redirection thread on exit...");
            if let Err(e) = handle.join() {
                eprintln!("Error joining Input-Redirection thread on exit: {:?}", e);
            } else {
                println!("Input-Redirection thread joined on exit.");
            }
        }
    }
}
