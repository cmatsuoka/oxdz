use std::cmp;
use format::{ProbeInfo, Format, Loader};
use format::st::StData;
use format::mk::{ModPatterns, ModInstrument};
use module::{Module, Sample};
use module::sample::SampleType;
use util::BinaryRead;
use ::*;


/// Soundtracker 15-instrument module loader
pub struct StLoader;

impl Loader for StLoader {
    fn name(&self) -> &'static str {
        "Soundtracker"
    }

    fn probe(&self, b: &[u8], player_id: &str) -> Result<ProbeInfo, Error> {

        player::check_accepted(player_id, "st")?;

        // Ultimate Soundtracker support based on the module format description
        // written by Michael Schwendt
        let mut ust = true;

        if b.len() < 600 {
            return Err(Error::Format(format!("file too short ({})", b.len())));
        }

        // check title
        if !test_name(b, 0, 20) {
            return Err(Error::Format("invalid title".to_owned()));
        }

        // check instruments
        let mut ofs = 20;
        let mut total_size = 0;
        for i in 0..15 {
            // Crepequs.mod has random values in first byte
            if !test_name(b, ofs + 1, 21) {
                return Err(Error::Format(format!("sample {} invalid instrument name", i)));
            }

            let size = b.read16b(ofs+22)?;
            if size > 0x8000 {
                return Err(Error::Format(format!("sample {} invalid instrument size {}", i, size)));
            }
            if b.read8(ofs+24)? != 0 {
                return Err(Error::Format(format!("sample {} has finetune", i)));
            }
            if b.read8(ofs+25)? > 0x40 {
                return Err(Error::Format(format!("sample {} invalid volume", i)));
            }
            let repeat = b.read16b(ofs+26)?;
            if repeat>>1 > size {
                return Err(Error::Format(format!("sample {} repeat > size", i)));
            }
            let replen = b.read16b(ofs+28)?;
            if replen > 0x8000 {
                return Err(Error::Format(format!("sample {} invalid replen", i)));
            }
            if size > 0 && repeat>>1 == size {
                return Err(Error::Format(format!("sample {} repeat > size", i)));
            }
            if size == 0 && repeat > 0 {
                return Err(Error::Format(format!("sample {} invalid repeat", i)));
            }

            ofs += 30;
            total_size += size * 2;

            // UST: Maximum sample length is 9999 bytes decimal, but 1387
            // words hexadecimal. Longest samples on original sample disk
            // ST-01 were 9900 bytes.
            if size > 0x1387 || repeat > 9999 || replen > 0x1387 {
                ust = false
            }
        }

        if total_size < 8 {
            return Err(Error::Format(format!("invalid total sample size {}", total_size)));
        }

        // check length
        let len = b.read8(470)?;
        if len == 0 || len > 0x7f {
            return Err(Error::Format(format!("invalid length {}", len)));
        }

        // check tempo setting
        let tempo = b.read8(471)?;
        if tempo < 0x20 {
            return Err(Error::Format(format!("invalid initial tempo {}", tempo)));
        }

        // UST: The byte at module offset 471 is BPM, not the song restart
        //      The default for UST modules is 0x78 = 120 BPM = 48 Hz.
        // if tempo < 0x40 {    // should be 0x20
        //    ust = false
        // }

        // check orders
        let mut pat = 0;
        for i in 0..128 {
            let p = b.read8(472+i)?;
            if p > 0x7f {
                return Err(Error::Format(format!("invalid pattern number {} in orders", p)));
            }
            pat = cmp::max(pat, p)
        }
        pat += 1;

        // check patterns
        let mut cmd_used = 0_u32;
        for i in 0..pat as usize {
            for r in 0..64 {
                for c in 0..4 {
                    let ofs = 600 + 1024*i + 16*r + c*4;
                    let note = b.read16b(ofs)?;
                    if note & 0xf000 != 0 {
                        return Err(Error::Format("invalid event sample".to_owned()));
                    }
                    // check if note in table
                    if note != 0 && !NOTE_TABLE.contains(&note) {
                        return Err(Error::Format(format!("invalid note {}", note)));
                    }
                    // check invalid commands
                    let cmd = b.read8(ofs+2)? & 0x0f;
                    let cmdlo = b.read8(ofs+3)?;
                    if cmd != 0 {
                        cmd_used |= 1 << cmd
                    } else if cmdlo != 0 {
                        cmd_used |= 1
                    }

                    // UST: Only effects 1 (arpeggio) and 2 (pitchbend) are available
                    if cmd == 1 && cmdlo == 0 {  // unlikely arpeggio
                        ust = false
                    }
                    if cmd == 2 && (cmdlo & 0xf0) != 0 && (cmdlo & 0x0f) != 0 {  // bend up and down at same time
                        ust = false
                    }
                }
            }
        }

        if cmd_used & 0xfff9 != 0 {
            ust = false
        }

        let title = b.read_string(0,20)?;

        if ust {
            Ok(ProbeInfo{format: Format::Ust, title})
        } else {
            Ok(ProbeInfo{format: Format::St, title})
        }
    }

    fn load(self: Box<Self>, b: &[u8], info: ProbeInfo) -> Result<Module, Error> {

        if info.format != Format::St && info.format != Format::Ust {
            return Err(Error::Format("unsupported format".to_owned()));
        }

        let song_name = b.read_string(0, 20)?;

        // Load instruments
        let mut instruments: Vec<ModInstrument> = Vec::new();
        let mut samples: Vec<Sample> = Vec::new();
        for i in 0..15 {
            let ins = load_instrument(b, i)?;
            instruments.push(ins);
        }

        let song_length = b.read8(470)?;
        let tempo = b.read8(471)?;

        // Load orders
        let orders = b.slice(472, 128)?;

        let mut pat = 0_usize;
        orders[..song_length as usize].iter().for_each(|x| { pat = cmp::max(pat, *x as usize); } );
        pat += 1;

        // Load patterns
        let patterns = ModPatterns::from_slice(pat, b.slice(600, 1024*pat)?, 4)?;

        // Load samples
        let mut ofs = 600 + 1024*pat;
        for i in 0..15 {
            let size = instruments[i].size as usize * 2;
            let smp = load_sample(b.slice(ofs, size)?, ofs, i, &instruments[i]);
            samples.push(smp);
            ofs += size;
        }

        let mut data = StData{
            song_name,
            instruments,
            song_length,
            tempo,
            orders: [0; 128],
            patterns,
            samples,
        };

        data.orders.copy_from_slice(orders);

        let m = if info.format == Format::Ust {
            Module {
                format_id  : "st",
                description: "15 instrument module".to_owned(),
                creator    : "Ultimate Soundtracker".to_owned(),
                channels   : 4,
                player     : "ust",
                data       : Box::new(data),
            }
        } else {
            Module {
                format_id  : "st",
                description: "15 instrument module".to_owned(),
                creator    : "Soundtracker".to_owned(),
                channels   : 4,
                player     : "dst2",
                data       : Box::new(data),
            }
        };

        Ok(m)
    }
}

fn test_name(b: &[u8], ofs: usize, size: usize) -> bool {
    for x in b[ofs..ofs+size].iter() {
        if *x > 0x7f { return false }
        if *x > 0 && *x < 32 { return false }
    }
    true
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

lazy_static! {
    static ref NOTE_TABLE: Box<[u16; 37]> = Box::new([
        856, 808, 762, 720, 678, 640, 604, 570,
        538, 508, 480, 453, 428, 404, 381, 360,
        339, 320, 302, 285, 269, 254, 240, 226,
        214, 202, 190, 180, 170, 160, 151, 143,
        135, 127, 120, 113, 000
    ]);
}
