pub mod instrument;
pub mod sample;
pub mod event;

pub use self::sample::Sample;
pub use self::instrument::Instrument;
pub use self::instrument::SubInstrument;
pub use self::event::Event;

use std::any::Any;
use std::fmt;
use player::PlayerData;


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


// Module

#[derive(Debug)]
pub struct Module {
    pub title     : String,             // module title
    pub chn       : usize,              // number of channels
    pub speed     : usize,              // initial speed (frames per row)
    pub bpm       : usize,              // initial bpm (frame duration)
    pub instrument: Vec<Instrument>,
    pub sample    : Vec<Sample>,
    pub orders    : Box<Orders>,
    pub patterns  : Box<Patterns>,
}

impl Module {
    pub fn len(&self, song: usize) -> usize {
        self.orders.num(song)
    }
}
