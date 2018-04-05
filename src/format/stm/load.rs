use format::{ProbeInfo, Format, Loader};
use format::stm::{StmData, StmPatterns, StmInstrument};
use module::{Module, Sample};
use module::sample::SampleType;
use util::BinaryRead;
use ::*;

/// Scream Tracker 2 module loader
pub struct StmLoader;

impl Loader for StmLoader {
    fn name(&self) -> &'static str {
        "Scream Tracker 2"
    }

    fn probe(&self, b: &[u8], player_id: &str) -> Result<ProbeInfo, Error> {
        if b.len() < 1084 {
            return Err(Error::Format(format!("file too short ({})", b.len())));
        }

        player::check_accepted(player_id, "stm")?;

        let magic = b.read_string(20, 10)?;
        if magic == "!Scream!\x1a\x02" || magic == "BMOD2STM\x1a\x02" || magic == "WUZAMOD!\x1a\x02" || magic == "SWavePro\x1a\x02" {
            Ok(ProbeInfo{format: Format::Stm, title: b.read_string(0, 20)?})
        } else {
            Err(Error::Format(format!("bad magic {:?}", magic)))
        }
    }

    fn load(self: Box<Self>, b: &[u8], info: ProbeInfo) -> Result<Module, Error> {

        if info.format != Format::Stm {
            return Err(Error::Format("unsupported format".to_owned()));
        }

        let name = b.read_string(0, 20)?;

        let version_major = b.read8(30)?;
        let version_minor = b.read8(31)?;

        if version_major != 2 || version_minor < 21 {
            return Err(Error::Format(format!("unsupported version {}.{}", version_major, version_minor)));
        }

        let speed = b.read8(32)?;
        let num_patterns = b.read8(33)?;
        let global_vol = b.read8(34)?;
        let origin = b.read_string(20, 8)?;


        let mut instruments = Vec::<StmInstrument>::new();
        let mut samples = Vec::<Sample>::new();

        // Load instruments
        for i in 0..31 {
            let ins = load_instrument(b, i)?;
            instruments.push(ins);
        }

        // Load orders
        let orders = b.slice(1040, 128)?;

        // Load patterns
        let patterns = StmPatterns::from_slice(num_patterns as usize, b.slice(1168, 1024*num_patterns as usize)?)?;

        // Load samples
        let mut ofs = 1168 + 1024*num_patterns as usize;
        for i in 0..31 {
            let size = instruments[i].size as usize;
            let smp = load_sample(b.slice(ofs, size)?, ofs, i, &instruments[i]);
            samples.push(smp);
            ofs += size;
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
            format_id  : "stm",
            description: format!("Scream Tracker 2 STM"),
            creator    : match origin.as_ref() {
                             "!Scream!" => format!("Scream Tracker {}.{}", version_major, version_minor),
                             "BMOD2STM" => "BMOD2STM".to_owned(),
                             "WUZAMOD!" => "WUZAMOD".to_owned(),
                             "SWavePro" => "SWavePro".to_owned(),
                             _          => "unknown".to_owned(),
                         },
            channels   : 4,
            player     : "st2",
            data       : Box::new(data),
        };

        Ok(m)
    }
}

fn load_instrument(b: &[u8], i: usize) -> Result<StmInstrument, Error> {
    let mut ins = StmInstrument::new();

    let ofs = 48 + i * 32;
    ins.name = b.read_string(ofs, 12)?;
    ins.size = b.read16l(ofs + 16)?;
    ins.loop_start = b.read16l(ofs + 18)?;
    ins.loop_end = b.read16l(ofs + 20)?;
    ins.volume = b.read8(ofs + 22)?;
    ins.c2spd = b.read16l(ofs + 24)?;

    Ok(ins)
}

fn load_sample(b: &[u8], ofs: usize, i: usize, ins: &StmInstrument) -> Sample {
    let mut smp = Sample::new();

    smp.num = i + 1;
    smp.address = ofs as u32;
    smp.name = ins.name.to_owned();
    smp.rate = ins.c2spd as f64 / 8448.0;
    smp.size = ins.size as u32;
    if smp.size > 0 {
        smp.sample_type = SampleType::Sample8;
    }
    smp.store(b);

    smp
}

