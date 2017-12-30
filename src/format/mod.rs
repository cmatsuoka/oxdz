use super::module::Module;
use super::Error;

mod protracker;

pub trait ModuleFormat {
    fn name(&self) -> &'static str;
    fn load(&self, &[u8]) -> Result<Module, Error>;
}

pub fn list() -> Vec<Box<ModuleFormat>> {
    let mut v: Vec<Box<ModuleFormat>> = Vec::new();
    v.push(Box::new(protracker::Mod::new()));
    v
}


