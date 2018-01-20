use std::any::Any;
use std::cmp::max;
use Error;
use format::ModuleFormat;
use format::stm::{StmInstrument, StmEvent};
use module::{Module, Sample, Instrument, Orders, Patterns, Event};
use module::sample::SampleType;
use player::PlayerData;
use util::BinaryRead;

/// Scream Tracker 2 module loader
pub struct Stm;

impl Stm {
    fn load_instrument(&self, b: &[u8], i: usize) -> Result<(Instrument, Sample), Error> {
        let mut ins = Instrument::new();
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
        }

        let mut sub = StmInstrument::new();
        sub.smp_num = i;

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

impl ModuleFormat for Stm {
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
        let title = b.read_string(0, 20)?;

        let version_major = b.read8(30)?;
        let version_minor = b.read8(31)?;

        if version_major != 2 || version_minor < 21 {
            return Err(Error::Format("unsupported version"));
        }

        let tempo = b.read8(32)?;
        let num_patterns = b.read8(33)? as usize;
        let global_vol = b.read8(34)?;

        let mut ins_list = Vec::<Instrument>::new();
        let mut smp_list = Vec::<Sample>::new();

        // Load instruments
        for i in 0..31 {
            let (ins, smp) = try!(self.load_instrument(b, i));
            ins_list.push(ins);
            smp_list.push(smp);
        }

        // Load orders
        let ord = StmOrders::from_slice(b.slice(1040, 128)?);
        let len = ord.len(num_patterns);

        // Load patterns
        let patterns = StmPatterns::from_slice(num_patterns as usize, b.slice(1168, 1024*num_patterns)?)?;

        // Load samples
        let mut ofs = 1084 + 1024*num_patterns;
        for i in 0..31 {
            let size = smp_list[i].size as usize;
            if size > 0 {
                smp_list = try!(self.load_sample(b.slice(ofs, size)?, smp_list, i));
                ofs += size;
            }
        }

        let m = Module {
            format     : "stm",
            description: "Scream Tracker 2 STM".to_string(),
            player     : "st2",
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


pub struct StmPatterns {
    num : usize,
    data: Vec<StmEvent>,
}

impl StmPatterns {
    fn from_slice(num: usize, b: &[u8]) -> Result<Self, Error> {
        let mut pat = StmPatterns{
            num,
            data: Vec::new(),
        };

        for p in 0..num {
            for r in 0..64 {
                for c in 0..4 {
                    let ofs = p * 1024 + r * 16 + c * 4;
                    let e = StmEvent::from_slice(b.slice(ofs, 4)?);
                    pat.data.push(e);
                }
            }
        }

        Ok(pat)
    }

    pub fn event(&self, pat: u16, row: u16, chn: usize) -> &StmEvent {
        &self.data[pat as usize * 256 + row as usize * 4 + chn]
    }
}

impl Patterns for StmPatterns {
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
        e.ins  = raw.smp;
        e.vol  = raw.volume;
        e.fxt  = raw.cmd;
        e.fxp  = raw.infobyte;
        e
    }
}


struct StmOrders {
    orders: Vec<u8>,
    songs : Vec<u8>,  // vector of song entry points
}

impl StmOrders {
    fn from_slice(o: &[u8]) -> Self {
        StmOrders {
            orders: o.to_vec(),
            songs : Vec::new(),
        }
    }

    fn len(&self, pat: usize) -> usize {
        for (i, n) in self.orders.iter().enumerate() {
            if *n > pat as u8 {
                return i
            }
        }
        self.orders.len()
    }

    fn num_patterns(&self) -> usize {
        let mut num = 0;
        self.orders.iter().for_each(|x| num = max(*x as usize, num));
        num + 1
    }
}

impl Orders for StmOrders {
    fn num(&self, song: usize) -> usize {
        self.orders.len()
    }

    fn restart_position(&mut self) -> usize {
        0
    }

    fn pattern(&self, pos: usize) -> usize {
        self.orders[pos] as usize
    }

    fn next(&self, data: &mut PlayerData) -> usize {
        if data.pos < self.num(data.song) - 1 {
            data.pos += 1;
        }
        data.pos
    }

    fn prev(&self, data: &mut PlayerData) -> usize {
        if data.pos > 0 {
            data.pos -= 1;
        }
        data.pos
    }

    fn num_songs(&self) -> usize {
        self.songs.len()
    }

    fn next_song(&self, data: &mut PlayerData) -> usize {
        if data.song < self.num_songs() - 1 {
            data.song += 1;
        }
        data.song
    }

    fn prev_song(&self, data: &mut PlayerData) -> usize {
        if data.song > 0 {
            data.song -= 1;
        }
        data.song
    }
}
