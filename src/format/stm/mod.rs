pub mod load;

pub use self::load::*;

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
        e.note = b[0];
        e.volume = (b[1] & 0x07) | (b[2] & 0xf0) >> 1;
        e.smp = (b[1] & 0xf8) >> 3;
        e.cmd = b[2] & 0x0f;
        e.infobyte = b[3];
        e
    }
}

impl fmt::Display for StmEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let note = if self.note > 250 {
            "---".to_owned()
        } else {
            let n = ((self.note&0xf) + 12*(3+(self.note>>4))) as usize;
            format!("{}{}", NOTES[n%12], n/12)
        };

        let smp = if self.smp == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.smp)
        };

        let vol = if self.volume == 65 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.volume)
        };

        let cmd = if self.cmd == 0 {
            '.'
        } else {
            (64_u8 + self.cmd) as char
        };

        write!(f, "{} {} {} {}{:02X}", note, smp, vol, cmd, self.infobyte)
    }
}

