pub mod load;
pub mod player;

pub use self::load::*;
pub use self::player::*;

use std::any::Any;
use std::fmt;
use module::SubInstrument;
use util::NOTES;

/// StmInstrument defines extra instrument fields used in Protracker instruments.
#[derive(Debug)]
pub struct StmInstrument {
    pub smp_num : usize,
}

impl StmInstrument {
    pub fn new() -> Self {
        StmInstrument {
            smp_num : 0,
        }
    }
}

impl SubInstrument for StmInstrument {
    fn as_any(&self) -> &Any {
        self
    }

    fn sample_num(&self) -> usize {
        self.smp_num
    }
}


/// StmEvent defines the event format used in Scream Tracker 2 patterns.
#[derive(Default)]
pub struct StmEvent {
    pub note    : u8,
    pub volume  : u8,
    pub smp     : u8,
    pub cmd     : u8,
    pub infobyte: u8,
}

impl StmEvent {
    fn new() -> Self {
        Default::default()
    }

    fn from_slice(b: &[u8]) -> Self {
        let mut e = StmEvent::new();

        if b[0] != 251 && b[0] != 252 && b[0] != 253 {
            e.note = if b[0] == 255 {
                0
            } else {
                1 + (b[0]&0x0f) + 12*(3 + (b[0]>>4))
            };
            e.volume = (b[1] & 0x07) | (b[2] & 0xf0) >> 1;
            if e.volume > 0x40 {
                e.volume = 0x40;
            } else {
                e.volume += 1;
            }
            e.smp = (b[1] & 0xf8) >> 3;
            e.cmd = b[2] & 0x0f;
            e.infobyte = b[3];
        }
        e
    }
}

impl fmt::Display for StmEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let note = if self.note == 0 {
            "---".to_owned()
        } else {
            format!("{}{}", NOTES[self.note as usize % 12], self.note / 12)
        };

        let smp = if self.smp == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.smp)
        };

        let vol = if self.volume == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.volume)
        };

        write!(f, "{} {} {} {:02X}{:02X}", note, smp, vol, self.cmd, self.infobyte)
    }
}

