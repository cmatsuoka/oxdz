use byteorder::{ByteOrder, BigEndian};
use Error;
use ::*;

pub const NOTES: &'static [&'static str] = &[
    "C ", "C#", "D ", "D#", "E ", "F ", "F#", "G ", "G#", "A ", "A#", "B "
];

pub const C4_PAL_RATE : f64 = 8287.0;   // 7093789.2 / period (C4) * 2
pub const C4_NTSC_RATE: f64 = 8363.0;   // 7159090.5 / period (C4) * 2

// [Amiga] PAL color carrier frequency (PCCF) = 4.43361825 MHz
// [Amiga] CPU clock = 1.6 * PCCF = 7.0937892 MHz


#[macro_export]
macro_rules! try_option {
    ( $a: expr ) => {
        match $a {
            Some(v) => v,
            None    => return,
        }
    }
}

pub trait BinaryRead {
    fn read_string(&self, ofs: usize, size: usize) -> Result<String, Error>;
    fn read32b(&self, ofs: usize) -> Result<u32, Error>;
    fn read16b(&self, ofs: usize) -> Result<u16, Error>;
    fn read8(&self, ofs: usize) -> Result<u8, Error>;
    fn read8i(&self, ofs: usize) -> Result<i8, Error>;
    fn slice(&self, start: usize, size: usize) -> Result<&[u8], Error>;
}

impl<'a> BinaryRead for &'a [u8] {
    fn read_string(&self, ofs: usize, size: usize) -> Result<String, Error> {
        try!(check_buffer_size(&self, ofs + size));
        Ok(String::from_utf8_lossy(&self[ofs..ofs+size]).to_string().replace("\x00", " "))
    }

    fn read32b(&self, ofs: usize) -> Result<u32, Error> {
        try!(check_buffer_size(&self, ofs + 4));
        Ok(BigEndian::read_u32(&self[ofs..ofs+4]))
    }

    fn read16b(&self, ofs: usize) -> Result<u16, Error> {
        try!(check_buffer_size(&self, ofs + 2));
        Ok(BigEndian::read_u16(&self[ofs..ofs+2]))
    }

    fn read8(&self, ofs: usize) -> Result<u8, Error> {
        try!(check_buffer_size(&self, ofs + 1));
        Ok(self[ofs])
    }

    fn read8i(&self, ofs: usize) -> Result<i8, Error> {
        try!(check_buffer_size(&self, ofs + 1));
        Ok(self[ofs] as i8)
    }

    fn slice(&self, start: usize, size: usize) -> Result<&[u8], Error> {
        try!(check_buffer_size(&self, start + size));
        Ok(&self[start..start + size])
    }
}

fn check_buffer_size(b: &[u8], size: usize) -> Result<(), Error> {
    if size > b.len() {
        return Err(Error::Load("short read"))
    }
    Ok(())
}

pub fn period_to_note(period: u32) -> usize {
    if period == 0 {
        return 0
    }

    (12.0_f64 * (PERIOD_BASE / period as f64).log(2.0)).round() as usize + 1
}

pub fn note_to_period_mix(note: usize, bend: usize) -> f64 {
    let d = note as f64 + bend as f64 / 12800.0;
    PERIOD_BASE / 2.0_f64.powf(d / 12.0)
}

pub fn note_to_period(note: usize, finetune: isize, period_type: PeriodType) -> f64 {
    let d = note as f64 + finetune as f64 / 128_f64;
    match period_type {
        PeriodType::Linear => (240.0 - d) * 16.0,
        PeriodType::Amiga  => PERIOD_BASE / 2.0_f64.powf(d / 12.0),
    }
}
