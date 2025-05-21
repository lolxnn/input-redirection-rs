use crate::{constants::*, pad_state::PadState};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{net::UdpSocket, time::Instant};

pub struct Sender {
    sock: UdpSocket,
    target_ip: String,
    last_sent: Instant,
}

impl Sender {
    pub fn new(target_ip: String) -> Self {
        Self {
            sock: UdpSocket::bind("0.0.0.0:0").expect("UDP bind failed"),
            target_ip,
            last_sent: Instant::now(),
        }
    }

    pub fn set_target(&mut self, ip: String) {
        self.target_ip = ip;
    }

    /// Send a packet if at least 50 ms have elapsed since the last one.
    pub fn maybe_send(&mut self, st: &PadState) {
        if self.last_sent.elapsed().as_millis() < 50 {
            return;
        }
        self.last_sent = Instant::now();
        if self.target_ip.trim().is_empty() {
            return;
        }

        let mut buf = Vec::with_capacity(20);

        // Buttons
        buf.write_u32::<LittleEndian>(st.buttons).unwrap();
        // Touch (none)
        buf.write_u32::<LittleEndian>(0x0200_0000).unwrap();

        // Circle Pad (left stick)
        let x_cpad = (st.lx * CPAD_BOUND + CPAD_CENTER_OFFSET_INT as f32) as i32;
        let y_cpad = (st.ly * CPAD_BOUND + CPAD_CENTER_OFFSET_INT as f32) as i32;
        let circle_payload =
            ((y_cpad.clamp(0, 0xFFF) as u32) << 12) | (x_cpad.clamp(0, 0xFFF) as u32);
        buf.write_u32::<LittleEndian>(circle_payload).unwrap();

        // C-Stick (right stick) + IR buttons
        let rot = std::f32::consts::FRAC_1_SQRT_2;
        let rx = rot * (st.rx + st.ry) * CPP_BOUND + CPP_CENTER_OFFSET_INT as f32;
        let ry = rot * (st.ry - st.rx) * CPP_BOUND + CPP_CENTER_OFFSET_INT as f32;
        let c_stick_payload = (((ry as i32).clamp(0, 0xFF) as u32) << 24)
            | (((rx as i32).clamp(0, 0xFF) as u32) << 16)
            | ((st.ir_buttons as u32) << 8)
            | 0x81;
        buf.write_u32::<LittleEndian>(c_stick_payload).unwrap();

        // Special buttons (none)
        buf.write_u32::<LittleEndian>(0).unwrap();

        let _ = self
            .sock
            .send_to(&buf, (self.target_ip.as_str(), TARGET_PORT));
    }
}
