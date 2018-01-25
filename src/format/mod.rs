use module::{Module, Sample};
use ::*;

pub mod mk;
pub mod stm;

// Trait for module loader

pub trait Loader {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8]) -> Result<(), Error>;
    fn load(self: Box<Self>, &[u8]) -> Result<Module, Error>;
}


pub fn list() -> Vec<Box<Loader>> {
    vec![
        Box::new(mk::ModLoader),
        Box::new(stm::StmLoader),
    ]
}

pub fn load(b: &[u8]) -> Result<Module, Error> {

    for f in list() {
        println!("Probing format: {}", f.name());
        if f.probe(b).is_ok() {
            println!("Probe ok, load format");
            return f.load(b)
        }
    }

    Err(Error::Format("unsupported module format"))
}
