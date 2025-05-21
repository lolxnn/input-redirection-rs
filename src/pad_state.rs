use crate::constants::*;
use gilrs::Axis;

#[derive(Debug, Clone, Copy)]
pub struct PadState {
    pub lx: f32,
    pub ly: f32,
    pub rx: f32,
    pub ry: f32,
    pub buttons: u32,
    pub ir_buttons: u8,
}

impl PadState {
    pub fn new() -> Self {
        Self {
            lx: MINIMAL_NUDGE_LSTICK,
            ly: MINIMAL_NUDGE_LSTICK,
            rx: MINIMAL_NUDGE_RSTICK,
            ry: MINIMAL_NUDGE_RSTICK,
            buttons: 0xFFF,
            ir_buttons: 0,
        }
    }

    /// Apply dead-zone plus optional inversion to one stick axis.
    /// Returns `true` if the stored value was changed.
    pub fn apply_axis(&mut self, axis: Axis, value: f32, inverted: bool, deadzone: f32) -> bool {
        let in_val = if inverted { -value } else { value };

        let adjusted = if in_val.abs() < deadzone {
            match axis {
                Axis::RightStickX | Axis::RightStickY => {
                    if in_val >= 0.0 {
                        MINIMAL_NUDGE_RSTICK
                    } else {
                        -MINIMAL_NUDGE_RSTICK
                    }
                }
                _ => {
                    if in_val >= 0.0 {
                        MINIMAL_NUDGE_LSTICK
                    } else {
                        -MINIMAL_NUDGE_LSTICK
                    }
                }
            }
        } else {
            in_val
        };

        macro_rules! update {
            ($field:expr) => {{
                if ($field - adjusted).abs() > f32::EPSILON {
                    $field = adjusted;
                    true
                } else {
                    false
                }
            }};
        }

        match axis {
            Axis::LeftStickX => update!(self.lx),
            Axis::LeftStickY => update!(self.ly),
            Axis::RightStickX => update!(self.rx),
            Axis::RightStickY => update!(self.ry),
            _ => false,
        }
    }
}
