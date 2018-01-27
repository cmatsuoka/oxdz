pub mod sample;

pub use self::sample::Sample;

use std::fmt;
use std::any::Any;
use std::marker::{Sync, Send};
use util::NOTES;


// Module

pub struct Module {
    pub format_id  : &'static str,       // format identifier
    pub description: String,             // format description
    pub creator    : String,             // tracker name
    pub player     : &'static str,       // primary player for this format
    pub data       : Box<ModuleData>     //
}

impl Module {
    pub fn title(&self) -> &str {
        self.data.title()
    }

    pub fn channels(&self) -> usize {
        self.data.channels()
    }

    pub fn patterns(&self) -> usize {
        self.data.patterns()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn pattern_in_position(&self, pos: usize) -> Option<usize> {
        self.data.pattern_in_position(pos)
    }

    pub fn next_position(&self, pos: usize) -> usize {
        self.data.next_position(pos)
    }

    pub fn prev_position(&self, pos: usize) -> usize {
        self.data.prev_position(pos)
    }

    pub fn instruments(&self) -> Vec<String> {
        self.data.instruments()
    }

/*
    pub fn event(&self, num: usize, row: usize, chn: usize) -> Option<Event> {
        self.data.event(num, row, chn)
    }
*/

    pub fn rows(&self, pat: usize) -> usize {
        self.data.rows(pat)
    }

    pub fn samples(&self) -> &Vec<Sample> {
        self.data.samples()
    }
}

pub trait ModuleData: Send + Sync {
    fn as_any(&self) -> &Any;
    fn title(&self) -> &str;            // module title
    fn channels(&self) -> usize;        // number of channels
    fn patterns(&self) -> usize;        // number of patterns
    fn len(&self) -> usize;             // module length
    fn pattern_in_position(&self, usize) -> Option<usize>;
    fn next_position(&self, usize) -> usize;
    fn prev_position(&self, usize) -> usize;
    fn instruments(&self) -> Vec<String>;
    //fn event(&self, num: usize, row: usize, chn: usize) -> Option<Event>;
    fn rows(&self, pat: usize) -> usize;  // number of rows in pattern
    fn samples(&self) -> &Vec<Sample>;
}


/*
// Event

#[derive(Debug)]
pub struct Event {
    pub note: u8,  // note number (255 = no note, 60 = C5)
    pub ins : u8,  // instrument number (0 = no instrument)
    pub vol : u8,
    pub cmd : u8,
    pub info: u8,
}

impl Event {
    pub fn new() -> Self {
        Event {
            note: 0,
            ins : 0,
            vol : 0,
            fxt : 0,
            fxp : 0,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let note = if self.note == 255 {
            "---".to_owned()
        } else {
            format!("{}{}", NOTES[self.note as usize % 12], self.note / 12)
        };

        let ins = if self.ins == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.ins)
        };

        let vol = if self.vol == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.vol - 1)
        };

        write!(f, "{} {} {} {:02X}{:02X}", note, ins, vol, self.cmd, self.info)
    }
}
*/
