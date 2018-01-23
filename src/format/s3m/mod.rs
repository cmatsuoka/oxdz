use std::any::Any;
use module::SubInstrument;


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
    smp_num    : usize,
    pub typ    : u8,
    pub c2spd  : i32,   
    pub vol    : i8,
}

impl SubInstrument for S3mInstrument {
    fn as_any(&self) -> &Any {
        self
    }

    fn sample_num(&self) -> usize {
        self.smp_num
    }
}

