pub mod sample;

use self::sample::Sample;
use super::format;
use super::Error;

pub struct Module<'a> {
    title : String,
    sample: Vec<Sample<'a>>,
}

impl<'a> Module<'a> {
    pub fn new() -> Self {
        Module {
            title : "".to_owned(),
            sample: Vec::new(),
        }
    }

    pub fn from_buffer(b: &[u8]) -> Result<Self, Error> {
        let mut m = Self::new();

        for f in format::list() {
            let module = try!((*f).load(b));
            println!("module: {}", module.title);
        }

        Ok(m)
    }
}
