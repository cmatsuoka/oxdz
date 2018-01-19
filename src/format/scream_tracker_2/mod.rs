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


/// StmEvent defines the event format used in Protracker patterns.
pub struct StmEvent {
    pub note    : u8,
    pub volume  : u8,
    pub smp     : u8,
    pub cmd     : u8,
    pub infobyte: u8,
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

