use std::any::Any;
use std::fmt::Debug;
use ::*;

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


pub trait Instrument: Debug + Send + Sync {
    fn as_any(&self) -> &Any;
    fn num(&self) -> usize;
    fn name(&self) -> &str;
    fn volume(&self) -> usize;
}

pub struct Keymap<T> {
    map: [T; MAX_KEYS]
}

impl<T: fmt::Debug> fmt::Debug for Keymap<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.map[..].fmt(formatter)
    }
}
