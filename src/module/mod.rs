pub mod event;
pub mod sample;

pub use self::sample::Sample;

use std::any::Any;
use std::marker::{Sync, Send};
use util::MemOpExt;


// Module

pub struct Module {
    pub format_id  : &'static str,      // format identifier
    pub description: String,            // format description
    pub creator    : String,            // tracker name
    pub channels   : usize,             // number of mixer channels
    pub player     : &'static str,      // primary player for this format
    pub data       : Box<ModuleData>    //
}

impl Module {
    pub fn title(&self) -> &str {
        self.data.title()
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

    pub fn rows(&self, pat: usize) -> usize {
        self.data.rows(pat)
    }

    pub fn pattern_data(&self, pat: usize, mut buffer: &mut [u8]) -> usize {
        let length = buffer.len();
        buffer[..].fill(0, length);
        self.data.pattern_data(pat, length / 6, &mut buffer)
    }

    pub fn samples(&self) -> Vec<Sample> {
        self.data.samples()
    }
}

pub trait ModuleData: Send + Sync {
    fn as_any(&self) -> &Any;
    fn title(&self) -> &str;            // module title
    fn patterns(&self) -> usize;        // number of patterns
    fn len(&self) -> usize;             // module length
    fn pattern_in_position(&self, usize) -> Option<usize>;
    fn next_position(&self, usize) -> usize;
    fn prev_position(&self, usize) -> usize;
    fn instruments(&self) -> Vec<String>;
    fn rows(&self, pat: usize) -> usize;  // number of rows in pattern
    fn pattern_data(&self, pat: usize, num: usize, buffer: &mut [u8]) -> usize;
    fn samples(&self) -> Vec<Sample>;
}
