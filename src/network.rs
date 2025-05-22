use crate::{constants::*, pad_state::PadState};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{io::Cursor, net::UdpSocket, time::Instant};

pub struct Sender {
    sock: UdpSocket,
    target_ip: String,
    last_sent: Instant,
}

impl Sender {
    pub fn new(target_ip: String) -> Self {
        Self {
            // Bind local address to avoid "address already in use" error
            sock: UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket"),
            target_ip,
            last_sent: Instant::now(),
        }
    }

    pub fn set_target(&mut self, ip: String) {
        self.target_ip = ip;
    }

    pub fn maybe_send(&mut self, st: &PadState) {
        if self.last_sent.elapsed().as_millis() < 50 || self.target_ip.trim().is_empty() {
            return;
        }

        self.last_sent = Instant::now();
        let mut buf = [0u8; 20];
        let mut cursor = Cursor::new(&mut buf[..]);

        let _ = cursor.write_u32::<LittleEndian>(st.buttons);
        let _ = cursor.write_u32::<LittleEndian>(0x0200_0000);

        let x_cpad = (st.lx * CPAD_BOUND + CPAD_CENTER_OFFSET_INT as f32) as i32;
        let y_cpad = (st.ly * CPAD_BOUND + CPAD_CENTER_OFFSET_INT as f32) as i32;
        let circle_payload = (clamp_u12(y_cpad) << 12) | clamp_u12(x_cpad);
        let _ = cursor.write_u32::<LittleEndian>(circle_payload);

        let rx = (st.rx + st.ry) * ROT_CPP_BOUND + CPP_CENTER_OFFSET_INT as f32;
        let ry = (st.ry - st.rx) * ROT_CPP_BOUND + CPP_CENTER_OFFSET_INT as f32;
        let c_stick_payload = (clamp_u8(ry as i32) << 24)
            | (clamp_u8(rx as i32) << 16)
            | ((st.ir_buttons as u32) << 8)
            | 0x81;
        let _ = cursor.write_u32::<LittleEndian>(c_stick_payload);

        let _ = cursor.write_u32::<LittleEndian>(0);

        let _ = self.sock.send_to(&buf, (&*self.target_ip, TARGET_PORT));
    }
}

#[inline]
fn clamp_u12(x: i32) -> u32 {
    x.clamp(0, 0xFFF) as u32
}

#[inline]
fn clamp_u8(x: i32) -> u32 {
    x.clamp(0, 0xFF) as u32
}
