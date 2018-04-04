pub mod load;

pub use self::load::*;

use std::any::Any;
use format::mk::{ModInstrument, ModPatterns};
use module::{ModuleData, Sample};
use ::*;


pub struct StData {
    pub song_name  : String,
    pub instruments: Vec<ModInstrument>,
    pub song_length: u8,
    pub tempo      : u8,
    pub orders     : [u8; 128],
    pub patterns   : ModPatterns,
    pub samples    : Vec<Sample>,
}

impl ModuleData for StData {
    fn as_any(&self) -> &Any {
        self
    }

    fn title(&self) -> &str {
        &self.song_name
    }

    fn patterns(&self) -> usize {
        self.patterns.num()
    }

    fn len(&self) -> usize {
        self.song_length as usize
    }

    fn pattern_in_position(&self, pos: usize) -> Option<usize> {
        if pos >= self.orders.len() {
            None
        } else {
            Some(self.orders[pos] as usize)
        }
    }

    fn instruments(&self) -> Vec<String> {
        self.instruments.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>()
    }

    fn rows(&self, pat: usize) -> usize {
        if pat >= self.patterns.num() {
            0
        } else {
            64
        }
    }

    fn pattern_data(&self, pat: usize, num: usize, mut buffer: &mut [u8]) -> usize {
        format::mk::get_mod_pattern(&self.patterns.data(), pat, 4, num, &mut buffer)
    }

    fn samples(&self) -> Vec<Sample> {
        self.samples.to_owned()
    }
}
