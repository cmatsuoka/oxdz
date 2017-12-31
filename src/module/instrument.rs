use std::slice;
use super::Sample;
use super::super::*;

#[derive(Debug)]
pub struct Instrument {
    pub num   : usize,
    pub name  : String,
    pub volume: usize,
    pub keymap: Keymap<SampleMap>,
}

pub struct Keymap<T> {
    map: [T; MAX_KEYS]
}

impl<T: fmt::Debug> fmt::Debug for Keymap<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.map[..].fmt(formatter)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SampleMap {
    sample_num: u32,
    transpose : isize,
}

impl SampleMap {
    pub fn new() -> Self {
        SampleMap {
            sample_num: 0,
            transpose : 0,
        }
    }
}

impl Instrument {
    pub fn new() -> Instrument {
        Instrument {
            num   : 0,
            name  : "".to_owned(),
            volume: 0,
            keymap: Keymap{map: [SampleMap::new(); MAX_KEYS]},
        }
    }
}

