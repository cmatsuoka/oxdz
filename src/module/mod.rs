pub mod instrument;
pub mod sample;
pub mod event;

pub use self::sample::Sample;
pub use self::instrument::Instrument;
pub use self::event::Event;

use std::any::Any;
use std::fmt;
use player::Player;

pub trait Orders {
    fn num(&self, usize) -> usize;
    fn restart_position(&mut self) -> usize;
    fn pattern(&self, &Player) -> usize;
    fn next(&self, &mut Player) -> usize;
    fn prev(&self, &mut Player) -> usize;
    fn num_songs(&self) -> usize;
    fn next_song(&self, &mut Player) -> usize;
    fn prev_song(&self, &mut Player) -> usize;
}

impl fmt::Debug for Orders {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "length: {}", self.num(0))  // FIXME: how to deal with other songs?
    }
}


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


pub trait PlayFrame {
    fn name(&self) -> &'static str;
    fn play(&self, &Player, &Module);
}

impl fmt::Debug for PlayFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "player: {}", self.name())
    }
}


#[derive(Debug)]
pub struct Module {
    pub title     : String,           // module title
    pub chn       : usize,            // number of channels
    pub speed     : usize,            // initial speed (frames per row)
    pub instrument: Vec<Instrument>,
    pub sample    : Vec<Sample>,
    pub orders    : Box<Orders>,
    pub patterns  : Box<Patterns>,
    pub playframe : Box<PlayFrame>,
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
            playframe : Box::new(EmptyPlay),
        }
    }

    pub fn play_frame(&self, player: &Player) {
        self.playframe.play(player, &self)
    }
}

struct EmptyOrders;

impl Orders for EmptyOrders {
    fn num(&self, _song: usize) -> usize { 0 }
    fn restart_position(&mut self) -> usize { 0 }
    fn pattern(&self, _player: &Player) -> usize { 0 }
    fn next(&self, _player: &mut Player) -> usize { 0 }
    fn prev(&self, _player: &mut Player) -> usize { 0 }
    fn num_songs(&self) -> usize { 0 }
    fn next_song(&self, _player: &mut Player) -> usize { 0 }
    fn prev_song(&self, _player: &mut Player) -> usize { 0 }
}

struct EmptyPatterns;

impl Patterns for EmptyPatterns {
    fn as_any(&self) -> &Any { self }
    fn num(&self) -> usize { 0 }
    fn len(&self, _pat: usize) -> usize { 0 }
    fn rows(&self, _pat: usize) -> usize { 0 }
    fn event(&self, _num: usize, _row: usize, _chn: usize) -> Event { Event::new() }
}

struct EmptyPlay;

impl PlayFrame for EmptyPlay {
    fn name(&self) -> &'static str { "" }
    fn play(&self, _player: &Player, _module: &Module) { }
}


