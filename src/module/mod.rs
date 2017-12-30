pub mod sample;

use std::path::Path;
use self::sample::Sample;
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

    pub fn from_path<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        let mut m = Self::new();
        Ok(m)
    }
}
