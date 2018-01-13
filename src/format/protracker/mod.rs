pub mod load;
pub mod player;

pub use self::load::*;
pub use self::player::*;

use std::any::Any;
use std::fmt;
use module::SubInstrument;
use util::{NOTES, period_to_note};

/// ModInstrument defines extra instrument fields used in Protracker instruments.
#[derive(Debug)]
pub struct ModInstrument {
    pub finetune: isize,
    pub smp_num : usize,
}

impl ModInstrument {
    pub fn new() -> Self {
        ModInstrument {
            finetune: 0,
            smp_num : 0,
        }
    }
}

impl SubInstrument for ModInstrument {
    fn as_any(&self) -> &Any {
        self
    }

    fn sample_num(&self) -> usize {
        self.smp_num
    }
}


/// ModEvent defines the event format used in Protracker patterns.
pub struct ModEvent {
    pub note : u8,
    pub ins  : u8,
    pub cmd  : u8,
    pub cmdlo: u8,
}

impl ModEvent {
    fn from_slice(b: &[u8]) -> Self {
        ModEvent {
            note : period_to_note((((b[0] & 0x0f) as u32) << 8) | b[1] as u32) as u8,
            ins  : (b[0] & 0xf0) | ((b[2] & 0xf0) >> 4),
            cmd  : b[2] & 0x0f,
            cmdlo: b[3],
        }
    }
}

impl fmt::Display for ModEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let note = if self.note == 0 {
            "---".to_owned()
        } else {
            format!("{}{}", NOTES[self.note as usize % 12], self.note / 12)
        };

        let ins = if self.ins == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.ins)
        };

        write!(f, "{} {} {:02X}{:02X}", note, ins, self.cmd, self.cmdlo)
    }
}

