// Network
pub const TARGET_PORT: u16 = 4950;

// Stick bounds & offsets
pub const CPAD_BOUND: f32 = 0x5D0 as f32;
pub const CPP_BOUND: f32 = 0x7F as f32;
pub const CPAD_CENTER_OFFSET_INT: i32 = 0x800;
pub const CPP_CENTER_OFFSET_INT: i32 = 0x80;

// Tiny “nudge” values
pub const MINIMAL_NUDGE_LSTICK: f32 = 0.001;
pub const MINIMAL_NUDGE_RSTICK: f32 = 0.008;

// HID button bits
pub mod hid_bits {
    pub const A: u32 = 0;
    pub const B: u32 = 1;
    pub const SELECT: u32 = 2;
    pub const START: u32 = 3;
    pub const DRIGHT: u32 = 4;
    pub const DLEFT: u32 = 5;
    pub const DUP: u32 = 6;
    pub const DDOWN: u32 = 7;
    pub const R: u32 = 8;
    pub const L: u32 = 9;
    pub const X: u32 = 10;
    pub const Y: u32 = 11;
}

// IR button bits
pub mod ir_bits {
    pub const ZR: u8 = 1;
    pub const ZL: u8 = 2;
}
