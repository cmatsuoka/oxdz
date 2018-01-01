pub mod instrument;
pub mod sample;
pub mod event;

pub use self::sample::Sample;
pub use self::instrument::Instrument;
pub use self::event::Event;

use std::fmt;

pub trait Orders {
    fn num(&self) -> usize;
    fn restart(&mut self) -> usize;
    fn current(&self) -> usize;
    fn pattern(&self) -> usize;
    fn set(&mut self, usize) -> usize;
    fn next(&mut self) -> usize;
    fn prev(&mut self) -> usize;
    fn current_song(&self) -> usize;
    fn set_song(&mut self, usize) -> usize;
}

impl fmt::Debug for Orders {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "length: {}", self.num())
    }
}


pub trait Patterns {
    fn num(&self) -> usize;
    fn rows(&self, pat: usize) -> usize;
    fn event(&self, num: usize, row: usize, chn: usize) -> Event;
}

impl fmt::Debug for Patterns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "length: {}", self.num())
    }
}


#[derive(Debug)]
pub struct Module {
    pub title     : String,
    pub chn       : usize,
    pub instrument: Vec<Instrument>,
    pub sample    : Vec<Sample>,
    pub orders    : Box<Orders>,
    pub patterns  : Box<Patterns>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            title     : "".to_owned(),
            chn       : 0,
            instrument: Vec::new(),
            sample    : Vec::new(),
            orders    : Box::new(EmptyOrders),
            patterns  : Box::new(EmptyPatterns),
        }
    }
}

struct EmptyOrders;

impl Orders for EmptyOrders {
    fn num(&self) -> usize { 0 }
    fn restart(&mut self) -> usize { 0 }
    fn current(&self) -> usize { 0 }
    fn pattern(&self) -> usize { 0 }
    fn set(&mut self, _: usize) -> usize { 0 }
    fn next(&mut self) -> usize { 0 }
    fn prev(&mut self) -> usize { 0 }
    fn current_song(&self) -> usize { 0 }
    fn set_song(&mut self, _: usize) -> usize { 0 }
}

struct EmptyPatterns;

impl Patterns for EmptyPatterns {
    fn num(&self) -> usize { 0 }
    fn rows(&self, _pat: usize) -> usize { 0 }
    fn event(&self, _num: usize, _row: usize, _chn: usize) -> Event { Event::new() }
}
