pub mod instrument;
pub mod sample;

pub use self::sample::Sample;
pub use self::instrument::Instrument;

use Error;
use format;

#[derive(Debug)]
pub struct Module {
    pub title     : String,
    pub instrument: Vec<Instrument>,
    pub sample    : Vec<Sample>,
}

impl Module {
    pub fn new() -> Self {
        Module {
            title     : "".to_owned(),
            instrument: Vec::new(),
            sample    : Vec::new(),
        }
    }
}
