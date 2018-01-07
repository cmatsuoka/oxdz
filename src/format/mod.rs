use Error;
use module::Module;

mod protracker;

pub trait ModuleFormat {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8]) -> Result<(), Error>;
    fn load(self: Box<Self>, &[u8]) -> Result<Module, Error>;
}

pub fn list() -> Vec<Box<ModuleFormat>> {
    vec![
        Box::new(protracker::Mod::new())
    ]
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
