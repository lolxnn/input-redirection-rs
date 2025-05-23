use eframe::{App as EguiApp, CreationContext, egui};
use gilrs::{Axis, Button as GilrsButton, EventType, GamepadId, Gilrs};

use crate::{
    config::AppConfig,
    constants::{hid_bits, ir_bits},
    network::Sender,
    pad_state::PadState,
};

pub struct GuiApp {
    cfg: AppConfig,
    ip_buffer: String,
    gilrs: Gilrs,
    active_id: Option<GamepadId>,
    gamepad_name: String,
    state: PadState,
    sender: Sender,
}

impl GuiApp {
    pub fn new(_cc: &CreationContext) -> Self {
        let cfg = AppConfig::load().unwrap_or_default();
        let ip_buffer = cfg.target_ip.clone();
        let gilrs = Gilrs::new().unwrap();

        let (active_id, gamepad_name) = if let Some((id, gamepad)) = gilrs.gamepads().next() {
            (Some(id), gamepad.name().to_string())
        } else {
            (None, "Waiting for controller...".to_string())
        };

        let sender = Sender::new(cfg.target_ip.clone());

        Self {
            cfg,
            ip_buffer,
            gilrs,
            active_id,
            gamepad_name,
            state: PadState::new(),
            sender,
        }
    }

    fn axis_inverted(&self, axis: Axis) -> bool {
        match axis {
            Axis::LeftStickX => self.cfg.invert_lx,
            Axis::LeftStickY => self.cfg.invert_ly,
            Axis::RightStickX => self.cfg.invert_rx,
            Axis::RightStickY => self.cfg.invert_ry,
            _ => false,
        }
    }

    fn deadzone(&self, axis: Axis) -> f32 {
        match axis {
            Axis::LeftStickX | Axis::LeftStickY => self.cfg.deadzone_lstick,
            Axis::RightStickX | Axis::RightStickY => self.cfg.deadzone_rstick,
            _ => 0.0,
        }
    }

    fn handle_gilrs_event(&mut self, ev: gilrs::Event) {
        match ev.event {
            EventType::AxisChanged(axis, value, ..) => {
                if self
                    .state
                    .apply_axis(axis, value, self.axis_inverted(axis), self.deadzone(axis))
                {
                    self.sender.maybe_send(&self.state);
                }
            }
            EventType::ButtonPressed(btn, ..) => {
                self.update_button_state(btn, true);
                self.sender.maybe_send(&self.state);
            }
            EventType::ButtonReleased(btn, ..) => {
                self.update_button_state(btn, false);
                self.sender.maybe_send(&self.state);
            }
            EventType::Connected => {
                if self.active_id.is_none() {
                    self.active_id = Some(ev.id);
                    self.gamepad_name = self.gilrs.gamepad(ev.id).name().to_string();
                }
            }
            EventType::Disconnected => {
                if self.active_id == Some(ev.id) {
                    self.active_id = self.gilrs.gamepads().next().map(|(id, _)| id);
                }
            }
            _ => {}
        }
    }

    pub fn update_button_state(&mut self, b: GilrsButton, pressed: bool) {
        let btns = &mut self.state.buttons;
        let ir = &mut self.state.ir_buttons;

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
        match b {
            South => set_bit!(btns, hid_bits::B),
            East => set_bit!(btns, hid_bits::A),
            West => set_bit!(btns, hid_bits::X),
            North => set_bit!(btns, hid_bits::Y),
            DPadUp => set_bit!(btns, hid_bits::DUP),
            DPadDown => set_bit!(btns, hid_bits::DDOWN),
            DPadLeft => set_bit!(btns, hid_bits::DLEFT),
            DPadRight => set_bit!(btns, hid_bits::DRIGHT),
            Select => set_bit!(btns, hid_bits::SELECT),
            Start | Mode => set_bit!(btns, hid_bits::START),
            LeftTrigger => set_bit!(btns, hid_bits::L),
            RightTrigger => set_bit!(btns, hid_bits::R),
            LeftTrigger2 => {
                if pressed {
                    *ir |= 1 << ir_bits::ZL;
                } else {
                    *ir &= !(1 << ir_bits::ZL);
                }
            }
            RightTrigger2 => {
                if pressed {
                    *ir |= 1 << ir_bits::ZR;
                } else {
                    *ir &= !(1 << ir_bits::ZR);
                }
            }
            _ => {}
        }
    }
}

impl EguiApp for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // detect new/disconnected gamepads
        if self.active_id.is_none() {
            if let Some((id, gamepad)) = self.gilrs.gamepads().next() {
                self.active_id = Some(id);
                self.gamepad_name = gamepad.name().to_string();
            }
        } else if let Some(active) = self.active_id {
            if !self.gilrs.gamepads().any(|(id, _)| id == active) {
                eprintln!("Gamepad disconnected");
                self.active_id = None;
                self.gamepad_name = "Waiting for controller...".into();
            }
        }

        // process all pending events
        while let Some(ev) = self.gilrs.next_event() {
            if Some(ev.id) == self.active_id {
                self.handle_gilrs_event(ev);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("3DS Input-Redirection");
            ui.separator();

            // IP Address editor with commit on Enter or focus lost
            ui.horizontal(|ui| {
                ui.label("3DS IP Address:");
                let resp =
                    ui.add(egui::TextEdit::singleline(&mut self.ip_buffer).desired_width(120.0));
                if (resp.lost_focus() && resp.changed())
                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    self.cfg.target_ip = self.ip_buffer.clone();
                    let _ = self.cfg.save();
                    self.sender.set_target(self.cfg.target_ip.clone());
                }
            });

            ui.separator();

            // Dead-zone sliders
            let mut l_perc = self.cfg.deadzone_lstick * 100.0;
            if ui
                .add(
                    egui::Slider::new(&mut l_perc, 5.0..=50.0)
                        .text("Left-Stick Dead-zone")
                        .suffix("%")
                        .min_decimals(1)
                        .max_decimals(1),
                )
                .changed()
            {
                self.cfg.deadzone_lstick = (l_perc / 100.0).clamp(0.05, 0.50);
                let _ = self.cfg.save();
            }

            let mut r_perc = self.cfg.deadzone_rstick * 100.0;
            if ui
                .add(
                    egui::Slider::new(&mut r_perc, 5.0..=50.0)
                        .text("Right-Stick Dead-zone")
                        .suffix("%")
                        .min_decimals(1)
                        .max_decimals(1),
                )
                .changed()
            {
                self.cfg.deadzone_rstick = (r_perc / 100.0).clamp(0.05, 0.50);
                let _ = self.cfg.save();
            }

            ui.separator();

            // Inversion checkboxes
            if ui
                .checkbox(&mut self.cfg.invert_lx, "Invert Left-Stick X Axis")
                .changed()
                || ui
                    .checkbox(&mut self.cfg.invert_ly, "Invert Left-Stick Y Axis")
                    .changed()
                || ui
                    .checkbox(&mut self.cfg.invert_rx, "Invert Right-Stick X Axis")
                    .changed()
                || ui
                    .checkbox(&mut self.cfg.invert_ry, "Invert Right-Stick Y Axis")
                    .changed()
            {
                let _ = self.cfg.save();
            }

            ui.separator();
            ui.label(if let Some(_) = self.active_id {
                format!("Gamepad Active: {}", self.gamepad_name)
            } else {
                "No active gamepad".into()
            });
            ui.label(format!(
                "LX: {:.3}, LY: {:.3} (DZ {:.0}%)",
                self.state.lx,
                self.state.ly,
                self.cfg.deadzone_lstick * 100.0
            ));
            ui.label(format!(
                "RX: {:.3}, RY: {:.3} (DZ {:.0}%)",
                self.state.rx,
                self.state.ry,
                self.cfg.deadzone_rstick * 100.0
            ));
            ui.label(format!("HID Buttons: {:03X}", self.state.buttons));
            ui.label(format!("IR Buttons (ZL/ZR): {:02X}", self.state.ir_buttons));
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(16));
        self.sender.maybe_send(&self.state);
    }
}
