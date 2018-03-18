//pub mod load;

//pub use self::load::*;

use std::any::Any;
use module::{event, ModuleData, Sample};


pub struct SongHeaderTyp {
    sig        : String,
    name       : String,
    prog_name  : String,
    ver        : u16,
    header_size: i32,
    len        : u16,
    rep_s      : u16,
    ant_chn    : u16,
    ant_ptn    : u16,
    ant_instrs : u16,
    flags      : u16,
    def_tempo  : u16, 
    def_speed  : u16,
    song_tab   : [u8; 256],
}

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

pub struct InstrHeaderTyp {
    instr_size  : i32,
    name        : String,
    typ         : u8,
    ant_samp    : u16,
    sample_size : i32,
    ta          : [u8; 96],
    env_vp      : [[i16; 2]; 12],   
    env_pp      : [[i16; 2]; 12],   
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
    samp        : [SampleHeaderTyp; 32],
}

pub struct PatternHeaderTyp {
    pattern_header_size: i32,
    typ                : u8,
    patt_len           : u16,
    data_len           : u16,
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

    fn samples(&self) -> &Vec<Sample> {
        &self.samples
    }
}
