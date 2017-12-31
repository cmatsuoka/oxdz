use super::module::Module;
use super::Error;

mod protracker;

pub trait ModuleFormat {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8]) -> Result<(), Error>;
    fn load(self: Box<Self>, &[u8]) -> Result<Module, Error>;
}

pub fn list() -> Vec<Box<ModuleFormat>> {
    let mut v: Vec<Box<ModuleFormat>> = Vec::new();
    v.push(Box::new(protracker::Mod::new()));
    v
}

pub fn load_module(b: &[u8]) -> Result<Module, Error> {

    for f in list() {
        println!("Probing format: {}", f.name());
        if f.probe(b).is_ok() {
            println!("Probe ok, load format");
            return f.load(b)
        }
    }

    Err(Error::Format("unsupported module format"))
}
