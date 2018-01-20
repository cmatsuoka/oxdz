use std::any::Any;
use Error;
use format::{ModuleFormat, StdOrders};
use format::mk::{ModEvent, ModInstrument};
use module::{Module, Sample, Instrument, Patterns, Event};
use module::sample::SampleType;
use util::{self, BinaryRead};

/// Protracker module loader
pub struct Mod;

impl Mod {
    fn load_instrument(&self, b: &[u8], i: usize) -> Result<(Instrument, Sample), Error> {
        let mut ins = Instrument::new();
        let mut smp = Sample::new();

        let ofs = 20 + i * 30;
        ins.num = i + 1;
        smp.num = i + 1;
        ins.name = b.read_string(ofs, 22)?;
        smp.name = ins.name.to_owned();

        smp.size = b.read16b(ofs + 22)? as usize * 2;
        smp.rate = 8287.0;
        ins.volume = b.read8(ofs + 25)? as usize;
        smp.loop_start = b.read16b(ofs + 26)? as usize * 2;
        let loop_size = b.read16b(ofs + 28)?;
        smp.loop_end = smp.loop_start + loop_size as usize * 2;
        smp.has_loop = loop_size > 1 && smp.loop_end >= 4;

        let mut sub = ModInstrument::new();
        sub.finetune = (((b.read8i(ofs + 24)? << 4) as isize) >> 4) * 16;
        sub.smp_num = i;

        smp.rate = util::C4_PAL_RATE;
        if smp.size > 0 {
            smp.sample_type = SampleType::Sample8;
        }

        ins.subins.push(Box::new(sub));
        Ok((ins, smp))
    }

    fn load_sample(&self, b: &[u8], mut smp_list: Vec<Sample>, i: usize) -> Result<Vec<Sample>, Error> {
        if i >= smp_list.len() {
            return Err(Error::Load("invalid sample number"))
        }
        smp_list[i].store(b);
        Ok(smp_list)
    }
}

impl ModuleFormat for Mod {
    fn name(&self) -> &'static str {
        "Protracker MOD"
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
        let title = b.read_string(0, 20)?;

        let mut ins_list = Vec::<Instrument>::new();
        let mut smp_list = Vec::<Sample>::new();

        // Load instruments
        for i in 0..31 {
            let (ins, smp) = try!(self.load_instrument(b, i));
            ins_list.push(ins);
            smp_list.push(smp);
        }

        // Load orders
        let len = b.read8(950)? as usize;
        let rst = b.read8(951)?;
        let ord = StdOrders::from_slice(rst, b.slice(952, len)?);
        let pat = ord.num_patterns();

        // Load patterns
        let patterns = ModPatterns::from_slice(pat, b.slice(1084, 1024*pat)?)?;

        // Load samples (sample size is set when loading instruments)
        let mut ofs = 1084 + 1024*pat;
        for i in 0..31 {
            let size = smp_list[i].size as usize;
            if size > 0 {
                smp_list = try!(self.load_sample(b.slice(ofs, size)?, smp_list, i));
                ofs += size;
            }
        }

        let m = Module {
            format     : "mod",
            description: "Protracker M.K.".to_string(),
            player     : "pt21",
            title      : title,
            chn        : 4,
            speed      : 6,
            tempo      : 125,
            instrument : ins_list,
            sample     : smp_list,
            orders     : Box::new(ord),
            patterns   : Box::new(patterns),
        };

        Ok(m)
    }
}


pub struct ModPatterns {
    num : usize,
    data: Vec<ModEvent>,
}

impl ModPatterns {
    fn from_slice(num: usize, b: &[u8]) -> Result<Self, Error> {
        let mut pat = ModPatterns{
            num,
            data: Vec::new(),
        };

        for p in 0..num {
            for r in 0..64 {
                for c in 0..4 {
                    let ofs = p * 1024 + r * 16 + c * 4;
                    let e = ModEvent::from_slice(b.slice(ofs, 4)?);
                    pat.data.push(e);
                }
            }
        }

        Ok(pat)
    }

    pub fn event(&self, pat: usize, row: u8, chn: usize) -> &ModEvent {
        &self.data[pat * 256 + row as usize * 4 + chn]
    }
}

impl Patterns for ModPatterns {
    fn as_any(&self) -> &Any {
        self
    }

    fn num(&self) -> usize {
        self.num 
    }

    fn len(&self, pat: usize) -> usize {
        if pat >= self.num {
            0
        } else {
            64
        }
    }

    fn rows(&self, pat: usize) -> usize {
        if pat >= self.num() {
            0
        } else {
            64
        }
    }

    fn event(&self, num: usize, row: usize, chn: usize) -> Event {
        let ofs = num * 256 + row * 4 + chn;
        let mut e = Event::new();
        if ofs >= self.data.len() {
            return e
        }
        let raw = &self.data[ofs];
        e.note = raw.note;
        e.ins  = raw.ins;
        e.fxt  = raw.cmd;
        e.fxp  = raw.cmdlo;
        e
    }
}
