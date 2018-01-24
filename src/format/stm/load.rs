use format::Loader;
use format::stm::{StmData, StmPatterns, StmInstrument};
use module::{Module, Sample};
use module::sample::SampleType;
use util::BinaryRead;
use ::*;

/// Scream Tracker 2 module loader
pub struct StmLoader;

impl StmLoader {
    fn load_instrument(&self, b: &[u8], i: usize) -> Result<(StmInstrument, Sample), Error> {
        let mut ins = StmInstrument::new();
        let mut smp = Sample::new();

        let ofs = 48 + i * 32;
        ins.num = i + 1;
        smp.num = i + 1;;
        ins.name = b.read_string(ofs, 12)?;
        smp.size = b.read16l(ofs + 16)? as usize;
        smp.loop_start = b.read16l(ofs + 18)? as usize;
        smp.loop_end = b.read16l(ofs + 20)? as usize;
        ins.volume = b.read8(ofs + 22)? as usize;
        smp.rate = b.read16l(ofs + 24)? as f64;

        if smp.loop_end == 0xffff {
            smp.loop_end = 0;
        } else if smp.loop_end > 0 {
            smp.has_loop = true;
        }

        if smp.size > 0 {
            smp.sample_type = SampleType::Sample8;
        }

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

impl Loader for StmLoader {
    fn name(&self) -> &'static str {
        "Scream Tracker 2 STM"
    }
  
    fn probe(&self, b: &[u8]) -> Result<(), Error> {
        if b.len() < 1084 {
            return Err(Error::Format("file too short"));
        }

        if b.read_string(20, 10)? == "!Scream!\x1a\x02" {
            Ok(())
        } else {
            Err(Error::Format("bad magic"))
        }
    }

    fn load(self: Box<Self>, b: &[u8]) -> Result<Module, Error> {
        let name = b.read_string(0, 20)?;

        let version_major = b.read8(30)?;
        let version_minor = b.read8(31)?;

        if version_major != 2 || version_minor < 21 {
            return Err(Error::Format("unsupported version"));
        }

        let speed = b.read8(32)?;
        let num_patterns = b.read8(33)?;
        let global_vol = b.read8(34)?;

        let mut instruments = Vec::<StmInstrument>::new();
        let mut samples = Vec::<Sample>::new();

        // Load instruments
        for i in 0..31 {
            let (ins, smp) = try!(self.load_instrument(b, i));
            instruments.push(ins);
            samples.push(smp);
        }

        // Load orders
        let orders = b.slice(1040, 128)?;

        // Load patterns
        let patterns = StmPatterns::from_slice(num_patterns as usize, b.slice(1168, 1024*num_patterns as usize)?)?;

        // Load samples
        let mut ofs = 1168 + 1024*num_patterns as usize;
        for i in 0..31 {
            let size = samples[i].size as usize;
            if size > 0 {
                samples = try!(self.load_sample(b.slice(ofs, size)?, samples, i));
                ofs += size;
            }
        }

        let mut data = StmData{
            name,
            speed,
            num_patterns,
            global_vol,
            instruments,
            orders: [0; 128],
            patterns,
            samples,
        };

        data.orders.copy_from_slice(orders);

        let m = Module {
            format     : "stm",
            description: "Scream Tracker 2 STM",
            player     : "st2",
            data       : Box::new(data),
        };

        Ok(m)
    }
}

