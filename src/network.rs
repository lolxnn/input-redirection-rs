use crate::{constants::*, pad_state::PadState};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{io::Cursor, net::UdpSocket, time::SystemTime}; // Added SystemTime

pub struct Sender {
    sock: UdpSocket,
    target_ip: String,
    // No longer need last_sent_state
}

impl Sender {
    pub fn new(target_ip: String) -> Self {
        Self {
            sock: UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket"),
            target_ip,
            // No last_sent_state initialization needed
        }
    }

    pub fn send_state(&mut self, st: &PadState) {
        if cfg!(debug_assertions) {
            println!(
                "{} Sending state (always): {:?}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                st
            );
        }

        let mut buf = [0u8; 20];
        let mut cursor = Cursor::new(&mut buf[..]);

        // Your existing battle-tested serialization logic
        let _ = cursor.write_u32::<LittleEndian>(st.buttons);
        let _ = cursor.write_u32::<LittleEndian>(0x0200_0000);

        let x_cpad = (st.lx * CPAD_BOUND + CPAD_CENTER_OFFSET_INT as f32) as i32;
        let y_cpad = (st.ly * CPAD_BOUND + CPAD_CENTER_OFFSET_INT as f32) as i32;
        let circle_payload = (clamp_u12(y_cpad) << 12) | clamp_u12(x_cpad);
        let _ = cursor.write_u32::<LittleEndian>(circle_payload);

        let calculated_rx = (st.rx + st.ry) * ROT_CPP_BOUND + CPP_CENTER_OFFSET_INT as f32;
        let calculated_ry = (st.ry - st.rx) * ROT_CPP_BOUND + CPP_CENTER_OFFSET_INT as f32;
        let c_stick_payload = (clamp_u8(calculated_ry as i32) << 24)
            | (clamp_u8(calculated_rx as i32) << 16)
            | ((st.ir_buttons as u32) << 8)
            | 0x81;
        let _ = cursor.write_u32::<LittleEndian>(c_stick_payload);

        let _ = cursor.write_u32::<LittleEndian>(0);

        // Attempt to send
        if let Err(e) = self.sock.send_to(&buf, (&*self.target_ip, TARGET_PORT)) {
            // Still good to log errors if they occur, even if we don't change behavior based on them here
            eprintln!(
                "{} Failed to send UDP packet to {}:{}: {}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                self.target_ip,
                TARGET_PORT,
                e
            );
        }
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
