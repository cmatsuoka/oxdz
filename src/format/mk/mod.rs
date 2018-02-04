pub mod load;

pub use self::load::*;

use std::any::Any;
use module::{event, ModuleData, Sample};
use util::BinaryRead;
use ::*;

mod fingerprint;


pub struct ModData {
    pub song_name: String,
    pub instruments: Vec<ModInstrument>,
    pub song_length: usize,
    pub restart: u8,  // Noisetracker restart
    pub orders: [u8; 128],
    pub magic: String,
    pub patterns: ModPatterns,
    pub samples: Vec<Sample>,
}

impl ModuleData for ModData {
    fn as_any(&self) -> &Any {
        self
    }

    fn title(&self) -> &str {
        &self.song_name
    }

    fn channels(&self) -> usize {
        4
    }

    fn patterns(&self) -> usize {
        self.patterns.num()
    }

    fn len(&self) -> usize {
        self.song_length
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
        if pat >= self.patterns.num() {
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
            let note = e.note & 0xfff;
            let ins = (((e.note & 0xf000) >> 8) | ((e.cmd as u16 & 0xf0) >> 4)) as u8;

            if note  != 0 { flags |= event::HAS_NOTE; buffer[ofs+1] = period_to_note(note) }
            if ins   != 0 { flags |= event::HAS_INS ; buffer[ofs+2] = ins }
            if e.cmd != 0 || e.cmdlo != 0 { flags |= event::HAS_CMD; buffer[ofs+4] = e.cmd; buffer[ofs+5] = e.cmdlo }
            buffer[ofs] = flags;

            i += 1;
        }
        i
    }

    fn samples(&self) -> &Vec<Sample> {
        &self.samples
    }
}


#[derive(Debug,Default)]
pub struct ModInstrument {
    pub name    : String,
    pub volume  : u8,
    pub finetune: u8,
    pub size    : u16,
    pub repeat  : u16,
    pub replen  : u16,
}

impl ModInstrument {
    pub fn new() -> Self {
        Default::default()
    }
}


/// ModEvent defines the event format used in Protracker patterns.
pub struct ModEvent {
    pub note : u16,
    pub cmd  : u8,
    pub cmdlo: u8,
}

impl ModEvent {
    fn from_slice(b: &[u8]) -> Self {
        ModEvent {
            note : ((b[0] as u16) << 8) | b[1] as u16,
            cmd  : b[2],
            cmdlo: b[3],
        }
    }
}


pub struct ModPatterns {
    num : usize,
    data: Vec<ModEvent>,
}

impl ModPatterns {
    fn from_slice(num: usize, b: &[u8]) -> Result<Self, Error> {
        let mut pat = ModPatterns{
            num,
            data: Vec::new(),
        };

        for p in 0..num {
            for r in 0..64 {
                for c in 0..4 {
                    let ofs = p * 1024 + r * 16 + c * 4;
                    let e = ModEvent::from_slice(b.slice(ofs, 4)?);
                    pat.data.push(e);
                }
            }
        }
        
        Ok(pat)
    }

    pub fn num(&self) -> usize {
        self.num
    }

    pub fn event(&self, pat: usize, row: u8, chn: usize) -> &ModEvent {
        &self.data[pat * 256 + row as usize * 4 + chn]
    }
}


pub fn period_to_note(period: u16) -> u8 {
    if period == 0 {
        return 0
    }

    (12.0_f64 * (PERIOD_BASE / period as f64).log(2.0)).round() as u8
}

