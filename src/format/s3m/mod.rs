pub mod load;

pub use self::load::*;

use std::any::Any;
use std::fmt;
use module::{ModuleData, Event, Sample};
use util::NOTES;

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
    pub instrum_ptr: Vec<usize>,
    pub pattern_ptr: Vec<usize>,
    pub ch_pan     : [u8; 32],
    pub instruments: Vec<S3mInstrument>,
    pub patterns   : S3mPatterns,
    pub samples    : Vec<Sample>,
}

impl ModuleData for S3mData {
    fn as_any(&self) -> &Any {
        self
    }

    fn title(&self) -> &str {
        &self.song_name
    }

    fn channels(&self) -> usize {
        let mut chn = 0;
        for i in 0..32 {
            if self.ch_settings[i] == 0xff {
                continue
            }
            chn = i
        }
        chn + 1
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

    fn event(&self, num: usize, row: usize, chn: usize) -> Option<Event> {
        if num >= self.pat_num as usize || row >= 64 || chn >= 4 {
           return None
        } else {
           let p = self.patterns.event(num as u16, row as u16, chn);
           Some(Event{
               note: p.note,
               ins : p.ins,
               vol : p.volume,
               fxt : p.cmd,
               fxp : p.info,
           })
        }
    }

    fn rows(&self, pat: usize) -> usize {
        if pat >= self.pat_num as usize {
            0
        } else {
            64
        }
    }

    fn samples(&self) -> &Vec<Sample> {
        &self.samples
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

#[derive(Debug)]
pub struct S3mInstrument {
    pub typ    : u8,
    pub c2spd  : u32,
    pub vol    : i8,
}


#[derive(Default)]
pub struct S3mEvent {
    pub note  : u8,
    pub volume: u8,
    pub ins   : u8,
    pub cmd   : u8,
    pub info  : u8,
}

impl S3mEvent {
    fn new() -> Self {
        Default::default()
    }
}

impl fmt::Display for S3mEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let note = if self.note > 250 {
            "---".to_owned()
        } else {
            let n = ((self.note&0xf) + 12*(3+(self.note>>4))) as usize;
            format!("{}{}", NOTES[n%12], n/12)
        };

        let ins = if self.ins == 0 {
            "--".to_owned()
        } else {
            format!("{:02X}", self.ins)
        };

        let vol = if self.volume == 65 {
            "--".to_owned()
        } else {
            format!("{:02X}", self.volume)
        };

        let cmd = if self.cmd == 0 {
            '.'
        } else {
            (64_u8 + self.cmd) as char
        };

        write!(f, "{} {} {} {}{:02X}", note, ins, vol, cmd, self.info)
    }
}


pub struct S3mPatterns {
    pub data: Vec<u8>,
}

impl S3mPatterns {
    pub fn event(&self, pat: u16, row: u16, chn: usize) -> S3mEvent {
        //&self.data[pat as usize * 256 + row as usize * 4 + chn]
        S3mEvent::new()
    }
}

