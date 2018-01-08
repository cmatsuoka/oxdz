use std::fmt;
use Error;
use module::Module;
use player::{PlayerData, Virtual};

mod protracker;

// Trait for module formats

pub trait ModuleFormat {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8]) -> Result<(), Error>;
    fn load(self: Box<Self>, &[u8]) -> Result<(Module, Box<FormatPlayer>), Error>;
}


// Trait for format-specific players

pub trait FormatPlayer {
    fn name(&self) -> &'static str;
    fn play(&mut self, &mut PlayerData, &Module, &mut Virtual);
}

impl fmt::Debug for FormatPlayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "player: {}", self.name())
    }
}


pub fn list() -> Vec<Box<ModuleFormat>> {
    vec![
        Box::new(protracker::Mod::new())
    ]
}

pub fn load(b: &[u8]) -> Result<(Module, Box<FormatPlayer>), Error> {

    for f in list() {
        println!("Probing format: {}", f.name());
        if f.probe(b).is_ok() {
            println!("Probe ok, load format");
            return f.load(b)
        }
    }

    Err(Error::Format("unsupported module format"))
}



