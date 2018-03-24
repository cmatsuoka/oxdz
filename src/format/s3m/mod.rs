pub mod load;

pub use self::load::*;

use std::any::Any;
use module::{event, ModuleData, Sample};

//                                S3M Module header
//          0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
//        ,---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---.
//  0000: | Song name, max 28 chars (end with NUL (0))                    |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0010: |                                               |1Ah|Typ| x | x |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0020: |OrdNum |InsNum |PatNum | Flags | Cwt/v | Ffi   |'S'|'C'|'R'|'M'|
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0030: |g.v|i.s|i.t|m.v|u.c|d.p| x | x | x | x | x | x | x | x |Special|
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0040: |Channel settings for 32 channels, 255=unused,+128=disabled     |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0050: |                                                               |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0060: |Orders; length=OrdNum (should be even)                         |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  xxx1: |Parapointers to instruments; length=InsNum*2                   |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  xxx2: |Parapointers to patterns; length=PatNum*2                      |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  xxx3: |Channel default pan positions                                  |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+

pub struct S3mData {
    pub song_name  : String,
    pub ord_num    : u16,
    pub ins_num    : u16,
    pub pat_num    : u16,
    pub flags      : u16,
    pub cwt_v      : u16,
    pub ffi        : u16,
    pub g_v        : u8,
    pub i_s        : u8,
    pub i_t        : u8,
    pub m_v        : u8,
    pub d_p        : u8,
    pub ch_settings: [u8; 32],
    pub orders     : Vec<u8>,
    pub instrum_pp : Vec<u32>,
    pub pattern_pp : Vec<u32>,
    pub ch_pan     : [u8; 32],
    pub instruments: Vec<S3mInstrument>,
    pub patterns   : Vec<S3mPattern>,
    pub samples    : Vec<Sample>,

    pub channels   : usize,
}

impl ModuleData for S3mData {
    fn as_any(&self) -> &Any {
        self
    }

    fn title(&self) -> &str {
        &self.song_name
    }

    fn patterns(&self) -> usize {
        self.pat_num as usize
    }

    fn len(&self) -> usize {
        self.ord_num as usize
    }

    fn pattern_in_position(&self, pos: usize) -> Option<usize> {
        if pos >= self.orders.len() {
            None
        } else {
            Some(self.orders[pos] as usize)
        }
    }

    fn next_position(&self, _pos: usize) -> usize {
        0
    }

    fn prev_position(&self, _pos: usize) -> usize {
        0
    }

    fn instruments(&self) -> Vec<String> {
        self.samples.iter().map(|x| x.name.to_owned()).collect::<Vec<String>>()
    }

    fn rows(&self, pat: usize) -> usize {
        if pat >= self.pat_num as usize {
            0
        } else {
            64
        }
    }

    fn pattern_data(&self, pat: usize, num: usize, buffer: &mut [u8]) -> usize {

        let p = &self.patterns[pat].data;

        let mut ch;
        let mut row = 0;
        let mut i = 2;
        loop {
            let b = p[i]; i += 1;
            if b == 0 { row += 1; continue }
            ch = (b & 0x1f) as usize;

            let index = row * self.channels + ch;
            if index >= num { break }
            let ofs = 6 * index;

            if b & 0x20 != 0 {
                buffer[ofs] |= event::HAS_NOTE | event::HAS_INS;
                buffer[ofs + 1] = (p[i]>>4)*12+(p[i]&0x0f); i += 1;
                buffer[ofs + 2] = p[i]; i += 1;
            }
            if b & 0x40 != 0 {
                buffer[ofs] |= event::HAS_VOL;
                buffer[ofs + 3] = p[i]; i += 1;
            }
            if b & 0x80 != 0 {
                buffer[ofs] |= event::HAS_CMD;
                buffer[ofs + 4] = p[i]; i += 1;
                buffer[ofs + 5] = p[i]; i += 1;
            }
        }
        num
    }

    fn samples(&self) -> Vec<Sample> {
        self.samples.to_owned()
    }
}


//                        Digiplayer/ST3 samplefileformat
//          0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
//        ,---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---.
//  0000: |[T]| Dos filename (12345678.ABC)                   |    MemSeg |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0010: |Length |HI:leng|LoopBeg|HI:LBeg|LoopEnd|HI:Lend|Vol| x |[P]|[F]|
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0020: |C2Spd  |HI:C2sp| x | x | x | x |Int:Gp |Int:512|Int:lastused   |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0030: | Sample name, 28 characters max... (incl. NUL)                 |
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  0040: | ...sample name...                             |'S'|'C'|'R'|'S'|
//        +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
//  xxxx: sampledata

#[derive(Debug, Default)]
pub struct S3mInstrument {
    pub typ     : u8,
    pub memseg  : u32,
    pub length  : u32,
    pub loop_beg: u32,
    pub loop_end: u32,
    pub vol     : i8,
    pub flags   : i8,
    pub c2spd   : u32,
    pub name    : String,
}

impl S3mInstrument {
    pub fn new() -> Self {
        Default::default()
    }
}


pub struct S3mPattern {
    pub size: usize,
    pub data: Vec<u8>,
}
