pub mod load;

pub use self::load::*;

use std::any::Any;
use std::fmt;
use module::{event, ModuleData, Sample};
use util::{NOTES, BinaryRead};
use ::*;


pub struct StmData {
    pub name: String,
    pub speed: u8,
    pub num_patterns: u8,
    pub global_vol: u8,
    pub instruments: Vec<StmInstrument>,
    pub orders: [u8; 128],
    pub patterns: StmPatterns,
    pub samples: Vec<Sample>,
}

impl ModuleData for StmData {
    fn as_any(&self) -> &Any {
        self
    }

    fn title(&self) -> &str {
        &self.name
    }

    fn channels(&self) -> usize {
        4
    }

    fn patterns(&self) -> usize {
        self.num_patterns as usize
    }

    fn len(&self) -> usize {
        for i in 0..128 {
            if self.orders[i] >= self.num_patterns {
                return i
            }
        }
        128
    }


    fn pattern_in_position(&self, pos: usize) -> Option<usize> {
        if pos >= self.orders.len() {
            None
        } else {
            Some(self.orders[pos] as usize)
        }
    }

    fn next_position(&self, _pos: usize) -> usize {
        0
    }

    fn prev_position(&self, _pos: usize) -> usize {
        0
    }

    fn instruments(&self) -> Vec<String> {
        self.instruments.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>()
    }

    fn rows(&self, pat: usize) -> usize {
        if pat >= self.num_patterns as usize {
            0
        } else {
            64
        }
    }

    fn pattern_data(&self, pat: usize, num: usize, buffer: &mut [u8]) -> usize {
        let mut i = 0;
        for _ in 0..num {
            let (row, ch) = (i / 4, i % 4);
            let ofs = i * 6;
            let e = &self.patterns.data[pat*256 + row*4 + ch];

            let mut flags = 0;
            if e.note != 255 { flags |= event::HAS_NOTE; buffer[ofs+1] = e.note }
            if e.smp  != 0   { flags |= event::HAS_INS ; buffer[ofs+2] = e.smp  }
            if e.cmd  != 0   { flags |= event::HAS_CMD ; buffer[ofs+4] = e.cmd; buffer[ofs+5] = e.infobyte }
            buffer[ofs] = flags;

            i += 1;
        }
        i
    }

    fn samples(&self) -> &Vec<Sample> {
        &self.samples
    }
}

/// StmInstrument defines extra instrument fields used in Protracker instruments.
#[derive(Debug,Default)]
pub struct StmInstrument {
    pub name      : String,
    pub volume    : u8,
    pub size      : u16,
    pub loop_start: u16,
    pub loop_end  : u16,
    pub c2spd     : u16,
}

impl StmInstrument {
    pub fn new() -> Self {
        Default::default()
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
            format!("{:02X}", self.smp)
        };

        let vol = if self.volume == 65 {
            "--".to_owned()
        } else {
            format!("{:02X}", self.volume)
        };

        let cmd = if self.cmd == 0 {
            '.'
        } else {
            (64_u8 + self.cmd) as char
        };

        write!(f, "{} {} {} {}{:02X}", note, smp, vol, cmd, self.infobyte)
    }
}


pub struct StmPatterns {
    data: Vec<StmEvent>,
}

impl StmPatterns {
    fn from_slice(num: usize, b: &[u8]) -> Result<Self, Error> {
        let mut pat = StmPatterns{
            data: Vec::new(),
        };

        for p in 0..num {
            for r in 0..64 {
                for c in 0..4 {
                    let ofs = p * 1024 + r * 16 + c * 4;
                    let e = StmEvent::from_slice(b.slice(ofs, 4)?);
                    pat.data.push(e);
                }
            }
        }

        Ok(pat)
    }

    pub fn event(&self, pat: u16, row: u16, chn: usize) -> &StmEvent {
        &self.data[pat as usize * 256 + row as usize * 4 + chn]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event() {
        let e = StmEvent::from_slice(&[255, 1, 128, 0]);
        assert_eq!(format!("{}", e), "--- -- -- .00");

        let e = StmEvent::from_slice(&[34, 113, 128, 0]);
        assert_eq!(format!("{}", e), "D 5 0E -- .00");

        let e = StmEvent::from_slice(&[52, 50, 100, 204]);
        assert_eq!(format!("{}", e), "E 6 06 32 DCC");

        let e = StmEvent::from_slice(&[50, 49, 128, 0]);
        assert_eq!(format!("{}", e), "D 6 06 -- .00");
    }
}
