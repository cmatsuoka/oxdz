use std::any::Any;
use std::cmp::max;
use Error;
use format::ModuleFormat;
use format::protracker::ModPlayer;
use module::{Module, Sample, Instrument, Orders, Patterns, Event};
use player::Player;
use util::{BinaryRead, period_to_note};

/// Protracker module loader
pub struct Mod {
    name: &'static str,
}

impl Mod {
    pub fn new() -> Self {
        Mod{name: "Protracker MOD"}
    }

    fn load_instrument(&self, b: &[u8], mut m: Module, i: usize) -> Result<Module, Error> {
        let mut ins = Instrument::new();
        let mut smp = Sample::new();

        let ofs = 20 + i * 30;
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

    fn load_sample(&self, b: &[u8], mut m: Module, i: usize) -> Result<Module, Error> {
        if i >= m.sample.len() {
            return Err(Error::Load("invalid sample number"))
        }
        m.sample[i].store(b);

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
        m.chn = 4;
        m.speed = 6;

        // Load instruments
        for i in 0..31 {
            m = try!(self.load_instrument(b, m, i));
        }

        // Load orders
        let len = b.read8(950)? as usize;
        let rst = b.read8(951)?;
        let ord = ModOrders::from_slice(rst, b.slice(952, len)?);
        let pat = ord.patterns();
        m.orders = Box::new(ord);

        // Load patterns
        let p = ModPatterns::from_slice(pat, b.slice(1084, 1024*pat)?)?;
        m.patterns = Box::new(p);

        // Load samples (sample size is set when loading instruments)
        let mut ofs = 1084 + 1024*pat;
        for i in 0..31 {
            let size = m.sample[i].size as usize;
            m = try!(self.load_sample(b.slice(ofs, size)?, m, i));
            ofs += size;
        }

        // Set frame player
        let player = ModPlayer::new();
        m.playframe = Box::new(player);

        Ok(m)
    }
}

struct ModEvent {
    note: u8,
    ins : u8,
    fxt : u8,
    fxp : u8,
}

impl ModEvent {
    fn from_slice(b: &[u8]) -> Self {
        ModEvent {
            note: period_to_note((((b[0] & 0x0f) as u32) << 8) | b[1] as u32) as u8,
            ins : (b[0] & 0xf0) | ((b[2] & 0xf0) >> 4),
            fxt : b[2] & 0x0f,
            fxp : b[3],
        }
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
        e.fxt  = raw.fxt;
        e.fxp  = raw.fxp;
        e
    }
}


struct ModOrders {
    rstpos: usize,
    orders: Vec<u8>,
    songs : Vec<u8>,  // vector of song entry points
}

impl ModOrders {
    fn from_slice(r: u8, o: &[u8]) -> Self {
        
        let mut r = r as usize;

        if r >= o.len() {
            r = 0;
        }

        ModOrders {
            rstpos: r,
            orders: o.to_vec(),
            songs : Vec::new(),
        }
    }

    fn patterns(&self) -> usize {
        let mut num = 0;
        self.orders.iter().for_each(|x| num = max(*x as usize, num));
        num 
    }
}

impl Orders for ModOrders {
    fn num(&self, song: usize) -> usize {
        self.orders.len()
    }

    fn restart_position(&mut self) -> usize {
        self.rstpos
    }

    fn pattern(&self, player: &Player) -> usize {
        let pos = player.position();
        self.orders[pos] as usize
    }

    fn next(&self, player: &mut Player) -> usize {
        let mut pos = player.position();
        if pos < self.num(player.song()) - 1 {
            pos += 1;
        }
        player.set_position(pos);
        pos
    }

    fn prev(&self, player: &mut Player) -> usize {
        let mut pos = player.position();
        if pos > 0 {
            pos -= 1;
        }
        player.set_position(pos);
        pos
    }

    fn num_songs(&self) -> usize {
        self.songs.len()
    }

    fn next_song(&self, player: &mut Player) -> usize {
        let mut song = player.song();
        if song < self.num_songs() - 1 {
            song += 1;
        }
        player.set_song(song);
        song
    }

    fn prev_song(&self, player: &mut Player) -> usize {
        let mut song = player.song();
        if song > 0 {
            song -= 1;
        }
        player.set_song(song);
        song
    }
}
