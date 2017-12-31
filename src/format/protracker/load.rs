use byteorder::{ByteOrder, BigEndian};
use Error;
use format::ModuleFormat;
use module::{Module, Sample, Instrument};

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

    fn load_instrument(&self, b: &[u8], mut m: Module, i: usize) -> Result<Module, Error> {
        let mut ins = Instrument::new();
        let mut smp = Sample::new();

        let ofs = 20 + i * 30;
        ins.num = i + 1;
        ins.name = String::from_utf8_lossy(&b[ofs..ofs+22]).to_string();

        m.instrument.push(ins);
        m.sample.push(smp);

        Ok(m)
    }
}

impl ModuleFormat for Mod {
    fn name(&self) -> &'static str {
        self.name
    }
  
    fn probe(&self, b: &[u8]) -> Result<(), Error> {
        if b.len() < 1084 {
            return Err(Error::Format("file too short"));
        }

        if BigEndian::read_u32(&b[1080..1084]) == 0x4d2e4b2e {
            Ok(())
        } else {
            Err(Error::Format("bad magic"))
        }
    }

    fn load(self: Box<Self>, b: &[u8]) -> Result<Module, Error> {
        let mut m = Module::new();
        m.title = String::from_utf8_lossy(&b[..20]).to_string();

        for i in 0..31 {
            m = try!(self.load_instrument(b, m, i));
        }

        Ok(m)
    }

}
