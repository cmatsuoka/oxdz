use std::cmp;
use format::{ProbeInfo, Format, Loader};
use format::mk::{ModData, ModPatterns, ModInstrument};
use module::{Module, Sample};
use module::sample::SampleType;
use util::BinaryRead;
use ::*;

/// His Master's Noise module loader
pub struct FestLoader;

impl Loader for FestLoader {
    fn name(&self) -> &'static str {
        "His Master's Noise"
    }

    fn probe(&self, b: &[u8], player_id: &str) -> Result<ProbeInfo, Error> {
        if b.len() < 1084 {
            return Err(Error::Format(format!("file too short ({})", b.len())));
        }

        player::check_accepted(player_id, "fest")?;

        let magic = b.read_string(1080, 4)?;
        if magic == "FEST" {
            Ok(ProbeInfo{format: Format::Fest, title: b.read_string(0, 20)?})
        } else {
            Err(Error::Format(format!("bad magic {:?}", magic)))
        }
    }

    fn load(self: Box<Self>, b: &[u8], info: ProbeInfo) -> Result<Module, Error> {

        if info.format != Format::Fest {
            return Err(Error::Format("unsupported format".to_owned()));
        }

        let song_name = b.read_string(0, 20)?;

        let chn = 4;

        // Load instruments
        let mut instruments: Vec<ModInstrument> = Vec::new();
        let mut samples: Vec<Sample> = Vec::new();
        for i in 0..31 {
            let ins = load_instrument(b, i)?;
            instruments.push(ins);
        }

        // Load orders
        let song_length = b.read8(950)?;
        let restart = b.read8(951)?;
        let orders = b.slice(952, 128)?;
        let magic = b.read_string(1080, 4)?;

        let mut pat = 0_usize;
        orders[..song_length as usize].iter().for_each(|x| { pat = cmp::max(pat, *x as usize); } );
        pat += 1;

        // Load patterns
        let patterns = ModPatterns::from_slice(pat, b.slice(1084, 256*chn*pat)?, chn)?;

        // Load samples
        let mut ofs = 1084 + 256*chn*pat;
        for i in 0..31 {
            if &instruments[i].name[..4] == "Mupp" {
                let pat_num = instruments[i].name.as_bytes()[4] as usize;
                let pat_ofs = 1084 + 1024*pat_num;
                let smp = load_mupp(b.slice(pat_ofs, 1024)?, pat_ofs, i, pat_num);
                samples.push(smp);
            } else {
                let size = instruments[i].size as usize * 2;
                let smp = load_sample(b.slice(ofs, size)?, ofs, i, &instruments[i]);
                samples.push(smp);
                ofs += size;
            }
        }

        let mut data = ModData{
            song_name,
            instruments,
            song_length,
            restart,
            orders: [0; 128],
            magic,
            patterns,
            samples,
        };

        data.orders.copy_from_slice(orders);

        let m = Module {
            format_id  : "fest",
            description: "FEST module".to_owned(),
            creator    : "His Master's NoiseTracker".to_owned(),
            channels   : 4,
            player     : "hmn",
            data       : Box::new(data),
        };

        Ok(m)
    }
}


fn load_instrument(b: &[u8], i: usize) -> Result<ModInstrument, Error> {
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

fn load_mupp(b: &[u8], ofs: usize, i: usize, pat_num: usize) -> Sample {
    let mut smp = Sample::new();

    smp.num  = i + 1;
    smp.name = format!("Mupp @{}", pat_num);
    smp.address = ofs as u32;
    smp.size = 28*32;
    smp.sample_type = SampleType::Sample8;
    smp.store(b);

    smp
}

fn load_sample(b: &[u8], ofs: usize, i: usize, ins: &ModInstrument) -> Sample {
    let mut smp = Sample::new();

    smp.num  = i + 1;
    smp.name = ins.name.to_owned();
    smp.address = ofs as u32;
    smp.size = ins.size as u32 * 2;
    if smp.size > 0 {
        smp.sample_type = SampleType::Sample8;
    }
    smp.store(b);

    smp
}

