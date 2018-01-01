pub mod instrument;
pub mod sample;

pub use self::sample::Sample;
pub use self::instrument::Instrument;

use std::fmt;
use Error;
use format;

pub trait Order {
    fn len(&self) -> usize;
    fn restart(&mut self) -> usize;
    fn current(&self) -> usize;
    fn pattern(&self) -> usize;
    fn set(&mut self, usize) -> usize;
    fn next(&mut self) -> usize;
    fn prev(&mut self) -> usize;
    fn current_song(&self) -> usize;
    fn set_song(&mut self, usize) -> usize;
}

impl fmt::Debug for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "length: {}", self.len())
    }
}


#[derive(Debug)]
pub struct Module {
    pub title     : String,
    pub instrument: Vec<Instrument>,
    pub sample    : Vec<Sample>,
    pub orders    : Box<Order>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            title     : "".to_owned(),
            instrument: Vec::new(),
            sample    : Vec::new(),
            orders    : Box::new(EmptyOrders),
        }
    }
}

struct EmptyOrders;

impl Order for EmptyOrders {
    fn len(&self) -> usize { 0 }
    fn restart(&mut self) -> usize { 0 }
    fn current(&self) -> usize { 0 }
    fn pattern(&self) -> usize { 0 }
    fn set(&mut self, _: usize) -> usize { 0 }
    fn next(&mut self) -> usize { 0 }
    fn prev(&mut self) -> usize { 0 }
    fn current_song(&self) -> usize { 0 }
    fn set_song(&mut self, _: usize) -> usize { 0 }
}
