pub mod load;

pub use self::load::*;

use std::any::Any;
use module::{event, ModuleData, Sample};
use util::BinaryRead;
use ::*;


pub struct SongHeaderTyp {
    sig        : String,
    name       : String,
    prog_name  : String,
    ver        : u16,
    header_size: u32,
    len        : u16,
    rep_s      : u16,
    ant_chn    : u16,
    ant_ptn    : u16,
    ant_instrs : u16,
    flags      : u16,
    def_tempo  : u16, 
    def_speed  : u16,
    song_tab   : Vec<u8>,
}

impl SongHeaderTyp {
    pub fn from_slice(b: &[u8]) -> Result<Self, Error> {
        let sig = b.read_string(0, 17)?;
        let name = b.read_string(17, 20)?;
        let prog_name = b.read_string(38, 20)?;
        let ver = b.read16l(58)?;
        let header_size = b.read32l(60)?;
        let len = b.read16l(60 + 4)?;
        let rep_s = b.read16l(60 + 6)?;
        let ant_chn = b.read16l(60 + 8)?;
        let ant_ptn = b.read16l(60 + 10)?;
        let ant_instrs = b.read16l(60 + 12)?;
        let flags = b.read16l(60 + 14)?;
        let def_tempo = b.read16l(60 + 16)?;
        let def_speed = b.read16l(60 + 18)?;
        let mut song_tab: Vec<u8> = Vec::new();
        for i in 0..len as usize {
            song_tab.push(b.read8(60 + 20 + i)?);
        }

        Ok(SongHeaderTyp{
            sig,
            name,
            prog_name,
            ver,
            header_size,
            len,
            rep_s,
            ant_chn,
            ant_ptn,
            ant_instrs,
            flags,
            def_tempo,
            def_speed,
            song_tab,
        })
    }
}


#[derive(Default)]
pub struct SampleHeaderTyp {
    len    : i32,
    rep_s  : i32,
    rep_l  : i32,
    vol    : u8,
    fine   : i8,
    typ    : u8,
    pan    : u8,
    rel_ton: i8,
    skrap  : u8,
    name   : String,
}

impl SampleHeaderTyp {
    pub fn new() -> Self {
        Default::default()
    }
}


#[derive(Default)]
pub struct InstrHeaderTyp {
    instr_size  : u32,
    name        : String,
    typ         : u8,
    ant_samp    : u16,
    sample_size : i32,
    ta          : Vec<u8>, //[u8; 96],
    env_vp      : Vec<Vec<i16>>, //[[i16; 2]; 12],   
    env_pp      : Vec<Vec<i16>>, //[[i16; 2]; 12],   
    env_vp_ant  : u8,
    env_pp_ant  : u8,
    env_v_sust  : u8,
    env_v_rep_s : u8,
    env_v_rep_e : u8,
    env_p_sust  : u8,
    env_p_rep_s : u8,
    env_p_rep_e : u8,
    env_v_typ   : u8,
    env_p_typ   : u8,
    vib_type    : u8,
    vib_sweep   : u8,
    vib_depth   : u8,
    vib_rate    : u8,
    fade_out    : u16,
    //midi_on     : bool,
    //midi_channel: u8,
    //midi_program: i16,
    //midi_bend   : i16,
    //mute        : bool,
    samp        : Vec<SampleHeaderTyp>,
}

impl InstrHeaderTyp {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_slice(b: &[u8]) -> Result<Self, Error> {
        let mut ins = Self::new();
        ins.instr_size = b.read32l(0)?;
        ins.name =  b.read_string(4, 22)?;
        ins.typ = b.read8(26)?;
        ins.ant_samp = b.read16l(27)?;

        Ok(ins)
    }
}


#[derive(Default)]
pub struct TonTyp {
    pub ton    : u8,
    pub instr  : u8,
    pub vol    : u8,
    pub eff_typ: u8,
    pub eff    : u8,
}

impl TonTyp {
    pub fn new() -> Self {
        Default::default()
    }
}


pub struct PatternHeaderTyp {
    pattern_header_size: i32,
    typ                : u8,
    patt_len           : u16,
    data_len           : u16,
    num_chn            : usize,
    data               : Vec<TonTyp>,
}


impl PatternHeaderTyp {
    pub fn from_slice(num: usize, b: &[u8], num_chn: usize) -> Result<(Self, usize), Error> {
        let pattern_header_size = b.read32l(0)? as i32;
        let typ = b.read8(4)?;
        let patt_len = b.read16l(5)?;
        let data_len = b.read16l(7)?;

        let mut pat = PatternHeaderTyp{
            pattern_header_size,
            typ,
            patt_len,
            data_len,
            num_chn,
            data: Vec::new(),
        };

        let mut ofs = 9;
        for r in 0..patt_len as usize {
            for c in 0..num_chn {
                let mut e = TonTyp::new();
                let ton = b.read8(ofs)?;
                ofs += 1;
                let event = if ton & 0x80 != 0 {
                    // packed event
                    if ton & 0x01 != 0 {
                        e.ton = b.read8(ofs)?;
                        ofs += 1;
                    }
                    if ton & 0x02 != 0 {
                        e.instr = b.read8(ofs)?;
                        ofs += 1;
                    }
                    if ton & 0x04 != 0 {
                        e.vol = b.read8(ofs)?;
                        ofs += 1;
                    }
                    if ton & 0x08 != 0 {
                        e.eff_typ = b.read8(ofs)?;
                        ofs += 1;
                    }
                    if ton & 0x10 != 0 {
                        e.eff = b.read8(ofs)?;
                        ofs += 1;
                    }
                } else {
                    // unpacked event
                    e.ton = b.read8(ofs)?;
                    e.instr = b.read8(ofs + 1)?;
                    e.vol = b.read8(ofs + 2)?;
                    e.eff_typ = b.read8(ofs + 3)?;
                    e.eff = b.read8(ofs + 4)?;
                    ofs += 5
                };

                pat.data.push(e);
            }
        }

        Ok((pat, ofs))
    }

    pub fn data(&self) -> &Vec<TonTyp> {
        &self.data
    }

    pub fn event(&self, row: i16, chn: usize) -> &TonTyp {
        &self.data[row as usize*self.num_chn + chn]
    }
}



pub struct XmData {
    pub header     : SongHeaderTyp,
    pub instruments: Vec<InstrHeaderTyp>,
    pub patterns   : Vec<PatternHeaderTyp>,
    pub xm_samples : Vec<SampleHeaderTyp>,
    pub samples    : Vec<Sample>,
}

impl ModuleData for XmData {
    fn as_any(&self) -> &Any {
        self
    }

    fn title(&self) -> &str {
        &self.header.name
    }

    fn patterns(&self) -> usize {
        self.header.ant_ptn as usize
    }

    fn len(&self) -> usize {
        self.header.len as usize
    }

    fn pattern_in_position(&self, pos: usize) -> Option<usize> {
        if pos >= self.header.len as usize {
            None
        } else {
            Some(self.header.song_tab[pos] as usize)
        }
    }

    fn next_position(&self, _pos: usize) -> usize {
        0
    }

    fn prev_position(&self, _pos: usize) -> usize {
        0
    }

    fn instruments(&self) -> Vec<String> {
        self.instruments.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>()
    }

    fn rows(&self, pat: usize) -> usize {
        if pat >= self.header.ant_ptn as usize {
            0
        } else {
            self.patterns[pat].patt_len as usize
        }
    }

    fn pattern_data(&self, pat: usize, num: usize, buffer: &mut [u8]) -> usize {
        0 
    }

    fn samples(&self) -> Vec<Sample> {
        self.samples.to_owned()
    }
}
