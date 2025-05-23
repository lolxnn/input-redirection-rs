// poller_worker.rs
use gilrs::{Axis, Button as GilrsButton, EventType, GamepadId, Gilrs};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

// Assuming these modules are accessible from the crate root (e.g., `crate::constants`)
// If your project structure is different, you might need to adjust these paths.
use crate::{
    constants::{hid_bits, ir_bits},
    network::Sender,
    pad_state::PadState,
};

/// Configuration for the PollerWorker.
/// This struct holds the necessary configuration values that were previously
/// part of AppConfig and directly used by the polling logic.
#[derive(Clone, Debug)]
pub struct PollerConfig {
    pub target_ip: String,
    pub deadzone_lstick: f32,
    pub deadzone_rstick: f32,
    pub invert_lx: bool,
    pub invert_ly: bool,
    pub invert_rx: bool,
    pub invert_ry: bool,
}

/// PollerWorker handles gamepad event polling and state sending in a separate thread.
pub struct PollerWorker {
    gilrs: Gilrs,
    active_id: GamepadId,
    cfg: PollerConfig,
    state: PadState,
    sender: Sender,
    running: Arc<AtomicBool>,
}

impl PollerWorker {
    pub fn new(
        gilrs: Gilrs,
        active_id: GamepadId,
        config: PollerConfig,
        running: Arc<AtomicBool>,
    ) -> Self {
        let sender = Sender::new(config.target_ip.clone());
        let state = PadState::new();

        PollerWorker {
            gilrs,
            active_id,
            cfg: config,
            state,
            sender,
            running,
        }
    }

    /// Updates the button bitfields based on press/release events.
    /// The logic is identical to the original implementation in CliApp.
    fn update_button_state(&mut self, btn: GilrsButton, pressed: bool) {
        let hid = &mut self.state.buttons;
        let ir = &mut self.state.ir_buttons;

        // Macro for HID buttons (pressed clears bit, released sets bit)
        macro_rules! set_bit {
            ($map:expr, $bit:expr) => {
                if pressed {
                    *$map &= !(1 << $bit);
                } else {
                    *$map |= 1 << $bit;
                }
            };
        }

        use GilrsButton::*;
        match btn {
            South => set_bit!(hid, hid_bits::B),
            East => set_bit!(hid, hid_bits::A),
            West => set_bit!(hid, hid_bits::X),
            North => set_bit!(hid, hid_bits::Y),
            DPadUp => set_bit!(hid, hid_bits::DUP),
            DPadDown => set_bit!(hid, hid_bits::DDOWN),
            DPadLeft => set_bit!(hid, hid_bits::DLEFT),
            DPadRight => set_bit!(hid, hid_bits::DRIGHT),
            Select => set_bit!(hid, hid_bits::SELECT),
            Start | Mode => set_bit!(hid, hid_bits::START),
            LeftTrigger => set_bit!(hid, hid_bits::L),
            RightTrigger => set_bit!(hid, hid_bits::R),
            LeftTrigger2 => {
                // IR Button: pressed sets bit, released clears bit
                if pressed {
                    *ir |= 1 << ir_bits::ZL;
                } else {
                    *ir &= !(1 << ir_bits::ZL);
                }
            }
            RightTrigger2 => {
                // IR Button: pressed sets bit, released clears bit
                if pressed {
                    *ir |= 1 << ir_bits::ZR;
                } else {
                    *ir &= !(1 << ir_bits::ZR);
                }
            }
            _ => {}
        }
    }

    /// Returns whether the axis should be inverted based on the worker's configuration.
    #[inline]
    fn axis_inverted(&self, axis: Axis) -> bool {
        match axis {
            Axis::LeftStickX => self.cfg.invert_lx,
            Axis::LeftStickY => self.cfg.invert_ly,
            Axis::RightStickX => self.cfg.invert_rx,
            Axis::RightStickY => self.cfg.invert_ry,
            _ => false,
        }
    }

    /// Returns the deadzone value for the given axis based on the worker's configuration.
    #[inline]
    fn deadzone(&self, axis: Axis) -> f32 {
        match axis {
            Axis::LeftStickX | Axis::LeftStickY => self.cfg.deadzone_lstick,
            Axis::RightStickX | Axis::RightStickY => self.cfg.deadzone_rstick,
            _ => 0.0,
        }
    }

    /// Runs the main event polling and state sending loop.
    /// This method is intended to be run in a separate thread.
    pub fn run(&mut self) {
        while self.running.load(Ordering::SeqCst) {
            let mut event_processed_and_state_changed = false;

            // Block-wait for events with a timeout (original code used 5ms in the example)
            // This allows the loop to periodically check the `running` flag.
            if let Some(evt) = self
                .gilrs
                .next_event_blocking(Some(Duration::from_millis(16)))
            {
                if evt.id == self.active_id {
                    match evt.event {
                        EventType::AxisChanged(axis, value, _) => {
                            let deadzone_val = self.deadzone(axis);
                            let inv = self.axis_inverted(axis);
                            if self.state.apply_axis(axis, value, inv, deadzone_val) {
                                event_processed_and_state_changed = true;
                            }
                        }
                        EventType::ButtonPressed(b, _) => {
                            self.update_button_state(b, true);
                            event_processed_and_state_changed = true;
                        }
                        EventType::ButtonReleased(b, _) => {
                            self.update_button_state(b, false);
                            event_processed_and_state_changed = true;
                        }
                        EventType::Connected | EventType::Disconnected => {
                            println!("Input-Redirection: Gamepad {:?} event: {:?}", evt.id, evt.event);
                        }
                        _ => {} // Other event types are ignored
                    }

                    if event_processed_and_state_changed {
                        self.sender.send_state(&self.state);
                    }
                }
            }
            // Unconditionally send state to ensure regular updates, as per original logic.
            self.sender.send_state(&self.state);
        }
    }
}
