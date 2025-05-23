// cli_app.rs
use gilrs::Gilrs; // Gilrs is used here for initial discovery
use std::{
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self, JoinHandle},
};

// Assuming these modules are accessible. Adjust paths if necessary.
use crate::{
    config::AppConfig,
    poller_worker::{PollerConfig, PollerWorker}, // Import new structs
};

/// CLI application now primarily manages the PollerWorker thread.
pub struct CliApp {
    running_signal: Arc<AtomicBool>, // Signal for the worker to stop
    worker_handle: Option<JoinHandle<()>>, // To join the worker thread
}

impl CliApp {
    /// Initialize Gilrs, discover gamepad, configure and spawn PollerWorker.
    pub fn new() -> Self {
        let app_cfg = AppConfig::load().unwrap_or_default();

        // Initialize Gilrs to find the active gamepad.
        // This Gilrs instance will be moved to the PollerWorker.
        let gilrs_instance = Gilrs::new().expect("Failed to initialize Gilrs");
        let (active_id, gamepad) = gilrs_instance
            .gamepads()
            .next()
            .expect("No gamepad connected. Please connect a gamepad and try again.");

        println!("3DS Input Redirection - CLI by lolxnn and contributors");
        println!("----------------------------------------");
        println!("Using gamepad '{}' (id {:?})", gamepad.name(), active_id);
        println!("Target IP: {}", app_cfg.target_ip);
        println!("LStick Deadzone: {}", app_cfg.deadzone_lstick);
        println!("RStick Deadzone: {}", app_cfg.deadzone_rstick);
        println!("Invert LStick X: {}", app_cfg.invert_lx);
        println!("Invert LStick Y: {}", app_cfg.invert_ly);
        println!("Invert RStick X: {}", app_cfg.invert_rx);
        println!("Invert RStick Y: {}", app_cfg.invert_ry);

        let poller_config = PollerConfig {
            target_ip: app_cfg.target_ip.clone(),
            deadzone_lstick: app_cfg.deadzone_lstick,
            deadzone_rstick: app_cfg.deadzone_rstick,
            invert_lx: app_cfg.invert_lx,
            invert_ly: app_cfg.invert_ly,
            invert_rx: app_cfg.invert_rx,
            invert_ry: app_cfg.invert_ry,
        };

        let running_signal = Arc::new(AtomicBool::new(true));

        // Create the PollerWorker instance, moving the gilrs instance and passing config.
        let mut poller_worker = PollerWorker::new(
            gilrs_instance, // Gilrs instance is moved here
            active_id,
            poller_config,
            running_signal.clone(),
        );

        // Spawn the PollerWorker in a new thread.
        let worker_handle = thread::spawn(move || {
            poller_worker.run();
        });

        CliApp {
            running_signal,
            worker_handle: Some(worker_handle),
        }
    }

    /// Main application execution: sets up Ctrl+C handler and waits for PollerWorker.
    pub fn run(&mut self) -> anyhow::Result<()> {
        let r = self.running_signal.clone();
        ctrlc::set_handler(move || {
            if r.load(Ordering::SeqCst) {
                println!("\nCtrl+C pressed. Signaling PollerWorker to stop...");
                r.store(false, Ordering::SeqCst);
            } else {
                println!("\nCtrl+C pressed again. PollerWorker is already stopping.");
            }
        })?;

        println!("CLI app running. PollerWorker is active in a separate thread.");
        println!("Press Ctrl+C to stop.");

        // Wait for the PollerWorker thread to complete its execution.
        if let Some(handle) = self.worker_handle.take() {
            match handle.join() {
                Ok(_) => println!("PollerWorker thread joined successfully."),
                Err(e) => eprintln!("PollerWorker thread panicked: {:?}", e),
            }
        } else {
            eprintln!("Error: PollerWorker thread handle was already taken or not initialized.");
        }

        println!("\nStopped CLI app.");
        Ok(())
    }
}
