pub mod instrument;
pub mod sample;
pub mod event;

pub use self::sample::Sample;
pub use self::instrument::Instrument;
pub use self::instrument::SubInstrument;
pub use self::event::Event;

use std::any::Any;
use std::fmt;
use player::{PlayerData, Virtual};

// Handle
pub struct Format {
    pub module: Module,
    pub player: Box<FormatPlayer>,
}

impl Format {
    pub fn from(module: Module, player: Box<FormatPlayer>) -> Self {
        Format {
            module,
            player, 
        }
    }
}


// Orders

pub trait Orders {
    fn num(&self, usize) -> usize;
    fn restart_position(&mut self) -> usize;
    fn pattern(&self, &PlayerData) -> usize;
    fn next(&self, &mut PlayerData) -> usize;
    fn prev(&self, &mut PlayerData) -> usize;
    fn num_songs(&self) -> usize;
    fn next_song(&self, &mut PlayerData) -> usize;
    fn prev_song(&self, &mut PlayerData) -> usize;
}

impl fmt::Debug for Orders {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "length: {}", self.num(0))  // FIXME: how to deal with other songs?
    }
}


// Patterns

pub trait Patterns: Any {
    fn as_any(&self) -> &Any;
    fn num(&self) -> usize;
    fn len(&self, usize) -> usize;
    fn rows(&self, pat: usize) -> usize;
    fn event(&self, num: usize, row: usize, chn: usize) -> Event;
}

impl fmt::Debug for Patterns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "length: {}", self.num())
    }
}


// Frame player

pub trait FormatPlayer {
    fn name(&self) -> &'static str;
    fn play(&mut self, &mut PlayerData, &Module, &mut Virtual);
}

impl fmt::Debug for FormatPlayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "player: {}", self.name())
    }
}


// Module

#[derive(Debug)]
pub struct Module {
    pub title     : String,             // module title
    pub chn       : usize,              // number of channels
    pub speed     : usize,              // initial speed (frames per row)
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
            speed     : 6,
            instrument: Vec::new(),
            sample    : Vec::new(),
            orders    : Box::new(EmptyOrders),
            patterns  : Box::new(EmptyPatterns),
        }
    }
}

struct EmptyOrders;

impl Orders for EmptyOrders {
    fn num(&self, _song: usize) -> usize { 0 }
    fn restart_position(&mut self) -> usize { 0 }
    fn pattern(&self, _data: &PlayerData) -> usize { 0 }
    fn next(&self, _data: &mut PlayerData) -> usize { 0 }
    fn prev(&self, _data: &mut PlayerData) -> usize { 0 }
    fn num_songs(&self) -> usize { 0 }
    fn next_song(&self, _data: &mut PlayerData) -> usize { 0 }
    fn prev_song(&self, _data: &mut PlayerData) -> usize { 0 }
}

struct EmptyPatterns;

impl Patterns for EmptyPatterns {
    fn as_any(&self) -> &Any { self }
    fn num(&self) -> usize { 0 }
    fn len(&self, _pat: usize) -> usize { 0 }
    fn rows(&self, _pat: usize) -> usize { 0 }
    fn event(&self, _num: usize, _row: usize, _chn: usize) -> Event { Event::new() }
}
