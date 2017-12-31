use Error;
use format::ModuleFormat;
use module::{Module, Sample, Instrument};
use util::BinaryRead;

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

        let mut ofs = 20 + i * 30;
        ins.num = i + 1;
        smp.num = i + 1;
        ins.name = b.read_string(ofs, 22)?;
        smp.name = ins.name.to_owned();

        smp.size = b.read16b(ofs + 22)? as u32 * 2;
        smp.rate = 8287.0;
        ins.volume = b.read8(ofs + 25)? as usize;
        smp.loop_start = b.read16b(ofs + 26)? as u32 * 2;
        let loop_size = b.read16b(ofs + 28)?;
        smp.loop_end = smp.loop_start + loop_size as u32 * 2;
        smp.has_loop = loop_size > 1 && smp.loop_end >= 4;

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

        if b.read32b(1080)? == 0x4d2e4b2e {
            Ok(())
        } else {
            Err(Error::Format("bad magic"))
        }
    }

    fn load(self: Box<Self>, b: &[u8]) -> Result<Module, Error> {
        let mut m = Module::new();
        m.title = b.read_string(0, 20)?;

        for i in 0..31 {
            m = try!(self.load_instrument(b, m, i));
        }

        Ok(m)
    }

}
