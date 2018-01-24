pub mod sample;
pub mod event;

pub use self::sample::Sample;
pub use self::event::Event;

use std::any::Any;
use std::marker::{Sync, Send};


// Module

pub struct Module<'a> {
    pub format     : &'static str,       // format identifier
    pub description: &'a str,            // format description
    pub player     : &'static str,       // primary player for this format
    pub data       : Box<ModuleData>     //
}

impl<'a> Module<'a> {
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

    pub fn event(&self, num: usize, row: usize, chn: usize) -> Option<Event> {
        self.data.event(num, row, chn)
    }

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
    fn event(&self, num: usize, row: usize, chn: usize) -> Option<Event>;
    fn rows(&self, pat: usize) -> usize;  // number of rows in pattern
    fn samples(&self) -> &Vec<Sample>;
}

