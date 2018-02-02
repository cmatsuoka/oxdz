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
    fn load_instrument(&self, b: &[u8], i: usize) -> Result<ModInstrument, Error> {
        let mut ins = ModInstrument::new();

        let ofs = 20 + i * 30;
        ins.name = b.read_string(ofs, 22)?;
        ins.size = b.read16b(ofs + 22)?;
        ins.finetune = b.read8(ofs + 24)?;
        ins.volume = b.read8(ofs + 25)?;
        ins.repeat = b.read16b(ofs + 26)?;
        ins.replen = b.read16b(ofs + 28)?;

        Ok(ins)
    }

    fn load_sample(&self, b: &[u8], i: usize, ins: &ModInstrument) -> Sample {
        let mut smp = Sample::new();

        smp.num  = i + 1;
        smp.name = ins.name.to_owned();
        smp.size = ins.size as u32 * 2;
        smp.rate = util::C4_PAL_RATE;
        if smp.size > 0 {
            smp.sample_type = SampleType::Sample8;
        }
        smp.store(b);

        smp
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

        let magic = b.read_string(1080, 4)?;
        if magic == "M.K." || magic == "M!K!" || magic == "M&K!" || magic == "N.T." {
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
            let ins = self.load_instrument(b, i)?;
            instruments.push(ins);
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

        // Load samples
        let mut ofs = 1084 + 1024*pat;
        for i in 0..31 {
            let size = instruments[i].size as usize * 2;
            let smp = self.load_sample(b.slice(ofs, size)?, i, &instruments[i]);
            samples.push(smp);
            ofs += size;
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
            format_id  : "mod",
            description: "M.K.".to_owned(),
            creator    : "Protracker".to_owned(),  // TODO: tracker fingerprinting
            player     : "pt21",
            data       : Box::new(data),
        };

        Ok(m)
    }
}


struct Fingerprint;

impl Fingerprint {
    fn has_restart_value(b: &[u8]) -> Result<bool, Error> {
        let restart = b.read8(951)?;
        Ok(restart != 0)
    }

    fn has_noisetracker_byte(b: &[u8]) -> Result<bool, Error> {
        let restart = b.read8(951)?;
        Ok(restart == 0x78)
    }

    fn has_noisetracker_magic(b: &[u8]) -> Result<bool, Error> {
        let magic = b.read_string(1080, 4)?;
        Ok(magic == "M&K!" || magic == "N.T.")
    }

    fn has_protracker_magic(b: &[u8]) -> Result<bool, Error> {
        let magic = b.read_string(1080, 4)?;
        Ok(magic == "M!K!")
    }
}
