use super::super::module::Module;
use super::super::Error;
use super::ModuleFormat;

pub struct Mod {
    name: &'static str,
}

struct ModPattern<'a> {
    pub event: &'a [ModEvent; 64 * 4],    
}

struct ModEvent {
    pub note: u8,
    pub ins : u8,
    pub fxp : u8,
    pub fxt : u8,
}

impl Mod {
    pub fn new() -> Self {
        Mod{name: "Protracker MOD"}
    }
}

impl ModuleFormat for Mod {
    fn name(&self) -> &'static str {
        self.name
    }
  
    fn load(&self, b: &[u8]) -> Result<Module, Error> {
        let m = Module::new();
        Ok(m)
    }
}
