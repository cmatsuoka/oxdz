use std::cmp;
use format::{ProbeInfo, Format, Loader};
use format::mk::{ModData, ModPatterns, ModInstrument};
use format::mk::fingerprint::{Fingerprint, TrackerID};
use module::{Module, Sample};
use module::sample::SampleType;
use util::BinaryRead;
use ::*;

/// Amiga tracker module loader
pub struct ModLoader;

impl Loader for ModLoader {
    fn name(&self) -> &'static str {
        "Amiga Protracker/Compatible"
    }

    fn probe(&self, b: &[u8], player_id: &str) -> Result<ProbeInfo, Error> {
        if b.len() < 1084 {
            return Err(Error::Format(format!("file too short ({})", b.len())));
        }

        let magic = b.read32b(1080)?;
        if magic == magic4!('M','.','K','.') || magic == magic4!('M','!','K','!') || magic == magic4!('M','&','K','!') || magic == magic4!('N','S','M','S') {
            player::check_accepted(player_id, "m.k.")?;
            Ok(ProbeInfo{format: Format::Mk, title: b.read_string(0, 20)?})
        } else if magic == magic4!('6','C','H','N') || magic == magic4!('8','C','H','N') {
            player::check_accepted(player_id, "xchn")?;
            Ok(ProbeInfo{format: Format::Xchn, title: b.read_string(0, 20)?})
        } else if magic & 0xffff == magic4!('\0','\0','C','H') {
            let c1 = (magic >> 24) as u8 as char;
            let c2 = ((magic & 0xff0000) >> 16) as u8 as char;
            if c1.is_digit(10) && c2.is_digit(10) {
                player::check_accepted(player_id, "xxch")?;
                Ok(ProbeInfo{format: Format::Xxch, title: b.read_string(0, 20)?})
            } else {
                Err(Error::Format(format!("bad magic {:?}", magic)))
            }
        } else if magic == magic4!('F','L','T','4') || magic == magic4!('F','L','T','8') {
            player::check_accepted(player_id, "flt")?;
            Ok(ProbeInfo{format: Format::Flt, title: b.read_string(0, 20)?})
        } else {
            Err(Error::Format(format!("bad magic {:?}", magic)))
        }
    }

    fn load(self: Box<Self>, b: &[u8], info: ProbeInfo) -> Result<Module, Error> {

        if info.format != Format::Mk && info.format != Format::Xchn && info.format != Format::Xxch {
            return Err(Error::Format("unsupported format".to_owned()));
        }

        let song_name = b.read_string(0, 20)?;

        // Load instruments
        let mut instruments: Vec<ModInstrument> = Vec::new();
        let mut samples: Vec<Sample> = Vec::new();
        let mut smp_size = 0;
        for i in 0..31 {
            let ins = load_instrument(b, i)?;
            smp_size += ins.size as usize * 2;
            instruments.push(ins);
        }

        // Load orders
        let song_length = b.read8(950)?;
        let restart = b.read8(951)?;
        let orders = b.slice(952, 128)?;
        let magic = b.read_string(1080, 4)?;

        let mut chn = channels_from_magic(&magic);

        let mut pat = 0;
        orders[..song_length as usize].iter().for_each(|x| { pat = cmp::max(pat, *x as usize); } );
        pat += 1;

        let mut tracker_id = TrackerID::Unknown;
        let data_size = 1084 + 256*pat*chn + smp_size;

        // Test for Flextrax modules
        //
        // FlexTrax is a soundtracker for Atari Falcon030 compatible computers. FlexTrax supports the
        // standard MOD file format (up to eight channels) for compatibility reasons but also features
        // a new enhanced module format FLX. The FLX format is an extended version of the standard
        // MOD file format with support for real-time sound effects like reverb and delay.
        if data_size + 4 < b.len() {
            if b.read32b(data_size)? == magic4!('F','L','E','X') {
                tracker_id = TrackerID::FlexTrax;
            }
        }

        // Test for Mod's Grave WOW modules
        //
        // Stefan Danes <sdanes@marvels.hacktic.nl> said:
        // This weird format is identical to '8CHN' but still uses the 'M.K.' ID. You can only test
        // for WOW by calculating the size of the module for 8 channels and comparing this to the
        // actual module length. If it's equal, the module is an 8 channel WOW.

        if magic == "M.K." && (data_size + 1024*pat) == b.len() {
            chn = 8;
            tracker_id = TrackerID::ModsGrave;
        }

        // Load patterns
        let patterns = ModPatterns::from_slice(pat, b.slice(1084, 256*chn*pat)?, chn)?;

        // Load samples
        let mut ofs = 1084 + 256*chn*pat;
        for i in 0..31 {
            let size = instruments[i].size as usize * 2;
            let smp = load_sample(b.slice(ofs, size)?, ofs, i, &instruments[i]);
            samples.push(smp);
            ofs += size;
        }

        let mut data = ModData{
            song_name,
            instruments,
            song_length,
            restart,
            orders: [0; 128],
            magic : magic.clone(),
            patterns,
            samples,
        };

        data.orders.copy_from_slice(orders);

        if tracker_id == TrackerID::Unknown {
            tracker_id = Fingerprint::id(&data)
        }

        let (creator, mut player_id) = match tracker_id {
            TrackerID::Unknown            => ("unknown tracker",  "pt2"),
            TrackerID::Protracker         => ("Protracker",       "pt2"),
            TrackerID::Noisetracker       => ("Noisetracker",     "nt"),
            TrackerID::Soundtracker       => ("Soundtracker",     "pt2"),
            TrackerID::Screamtracker3     => ("Scream Tracker 3", "st3"),
            TrackerID::FastTracker        => ("Fast Tracker",     "ft"),
            TrackerID::FastTracker2       => ("Fast Tracker",     "ft2"),
            TrackerID::TakeTracker        => ("TakeTracker",      "ft2"),
            TrackerID::Octalyser          => ("Octalyser",        "ft"),
            TrackerID::DigitalTracker     => ("Digital Tracker",  "pt2"),
            TrackerID::ModsGrave          => ("Mod's Grave",      "ft"),
            TrackerID::FlexTrax           => ("FlexTrax",         "pt2"),
            TrackerID::OpenMPT            => ("OpenMPT",          "pt2"),
            TrackerID::Converted          => ("Converted",        "pt2"),
            TrackerID::ConvertedST        => ("Converted 15-ins", "nt"),
            TrackerID::UnknownOrConverted => ("Unknown tracker",  "pt2"),
            TrackerID::ProtrackerClone    => ("Protracker clone", "pt2"),
        };

        debug!("Tracker: {} => player: {}", creator, player_id);

        // sanity check
        if player_id == "pt2" || player_id == "nt" {
            if chn > 8 {
                player_id = "ft2"
            } else if chn > 4 {
                player_id = "ft"
            }
        }

        // set format ID
        let mut format_id = "m.k.";
        if tracker_id == TrackerID::FastTracker {
            if chn == 6 || chn == 8 {
                format_id = "xchn";
            }
        }

        let m = Module {
            format_id,
            description: format!("{} module ", magic),
            creator    : creator.to_owned(),
            channels   : chn,
            player     : player_id,
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

fn channels_from_magic(magic: &str) -> usize {
    if magic == "FLT8" {
        8
    } else {
        let m: Vec<char> = magic.chars().collect();
        if m[0].is_digit(10) && m[1].is_digit(10) && &magic[2..] == "CH" {
            ((m[0] as u8 - '0' as u8) * 10 + m[1] as u8 - '0' as u8) as usize
        } else if m[0].is_digit(10) && &magic[1..] == "CHN" {
            (m[0] as u8 - '0' as u8) as usize
        } else {
            4
        }
    }
}
