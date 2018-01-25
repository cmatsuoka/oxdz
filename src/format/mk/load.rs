use std::cmp;
use format::Loader;
use format::mk::{ModData, ModPatterns, ModInstrument};
use module::{Module, Sample};
use module::sample::SampleType;
use util::{self, BinaryRead};
use ::*;

/// Protracker module loader
pub struct ModLoader;

impl ModLoader {
    fn load_instrument(&self, b: &[u8], i: usize) -> Result<(ModInstrument, Sample), Error> {
        let mut ins = ModInstrument::new();
        let mut smp = Sample::new();

        let ofs = 20 + i * 30;
        ins.name = b.read_string(ofs, 22)?;
        smp.name = ins.name.to_owned();

        smp.size = b.read16b(ofs + 22)? as usize * 2;
        smp.rate = 8287.0;
        ins.volume = b.read8(ofs + 25)? as usize;
        smp.loop_start = b.read16b(ofs + 26)? as usize * 2;
        let loop_size = b.read16b(ofs + 28)?;
        smp.loop_end = smp.loop_start + loop_size as usize * 2;
        smp.has_loop = loop_size > 1 && smp.loop_end >= 4;
        ins.finetune = (((b.read8i(ofs + 24)? << 4) as isize) >> 4) * 16;

        smp.rate = util::C4_PAL_RATE;
        if smp.size > 0 {
            smp.sample_type = SampleType::Sample8;
        }

        smp.sanity_check();

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

impl Loader for ModLoader {
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
        let song_name = b.read_string(0, 20)?;

        // Load instruments
        let mut instruments: Vec<ModInstrument> = Vec::new();
        let mut samples: Vec<Sample> = Vec::new();
        for i in 0..31 {
            let (ins, smp) = try!(self.load_instrument(b, i));
            instruments.push(ins);
            samples.push(smp);
        }

        // Load orders
        let song_length = b.read8(950)? as usize;
        let restart = b.read8(951)?;
        let orders = b.slice(952, 128)?;
        let magic = b.slice(1080, 4)?;

        let mut pat = 0_usize;
        orders[..song_length].iter().for_each(|x| { pat = cmp::max(pat, *x as usize); } );
        pat += 1;

        // Load patterns
        let patterns = ModPatterns::from_slice(pat, b.slice(1084, 1024*pat)?)?;

        // Load samples (sample size is set when loading instruments)
        let mut ofs = 1084 + 1024*pat;
        for i in 0..31 {
            let size = samples[i].size as usize;
            if size > 0 {
                samples = try!(self.load_sample(b.slice(ofs, size)?, samples, i));
                ofs += size;
            }
        }

        let mut data = ModData{
            song_name,
            instruments,
            song_length,
            restart,
            orders: [0; 128],
            magic: [0; 4],
            patterns,
            samples,
        };

        data.orders.copy_from_slice(orders);
        data.magic.copy_from_slice(magic);

        let m = Module {
            format     : "mod",
            description: "Protracker M.K.",
            player     : "pt21",
            data       : Box::new(data),
        };

        Ok(m)
    }
}

