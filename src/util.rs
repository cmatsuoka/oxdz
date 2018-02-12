use std::ptr;
use byteorder::{ByteOrder, BigEndian, LittleEndian};
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

#[macro_export]
macro_rules! clamp {
    ( $a:ident, $min:expr, $max:expr ) => {
        if $a < $min {
            $a = $min
        } else if $a > $max {
            $a = $max
        }
    }
}


pub trait MemOpExt<T> {
    fn fill(&mut self, u8, usize);
}

impl<'a, T> MemOpExt<T> for [T] {
    fn fill(&mut self, val: u8, amt: usize) {
        unsafe { ptr::write_bytes(self.as_mut_ptr(), val, amt * std::mem::size_of::<T>() - 1); }
    }
}


pub trait BinaryRead {
    fn read_string(&self, ofs: usize, size: usize) -> Result<String, Error>;
    fn read32b(&self, ofs: usize) -> Result<u32, Error>;
    fn read16b(&self, ofs: usize) -> Result<u16, Error>;
    fn read32l(&self, ofs: usize) -> Result<u32, Error>;
    fn read16l(&self, ofs: usize) -> Result<u16, Error>;
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

    fn read32l(&self, ofs: usize) -> Result<u32, Error> {
        try!(check_buffer_size(&self, ofs + 4));
        Ok(LittleEndian::read_u32(&self[ofs..ofs+4]))
    }

    fn read16l(&self, ofs: usize) -> Result<u16, Error> {
        try!(check_buffer_size(&self, ofs + 2));
        Ok(LittleEndian::read_u16(&self[ofs..ofs+2]))
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

fn check_buffer_size(b: &[u8], end: usize) -> Result<(), Error> {
    if end > b.len() {
        return Err(Error::Load(format!("short read (want {} bytes, have {})", end, b.len())))
    }
    Ok(())
}

pub fn note_to_period_mix(note: usize, bend: isize) -> f64 {
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

pub fn period_to_bend(period: f64, note: usize, ptype: PeriodType) -> isize {
    if note == 0 {
        return 0;
    }

    match ptype {
        PeriodType::Linear => {
            (100.0_f64 * (8.0 * (((240 - note) << 4) as f64 - period))) as isize
        },
        PeriodType::Amiga  => {
            let d = note_to_period(note, 0, PeriodType::Amiga);
            (100.0_f64 * 1536.0 * (d / period).log(2.0)).round() as isize
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_ERROR: f64 = 10e-5;

    macro_rules! assert_delta {
        ($x:expr, $y:expr) => {
            assert!(($x - $y).abs() < MAX_ERROR);
        }
    }

    #[test]
    fn test_note_to_period_mix() {
        assert_delta!(note_to_period_mix(60, 1000), 426.072926);
        assert_delta!(note_to_period_mix(60, -1000), 429.935790);
    }

    #[test]
    fn test_note_to_period() {
        assert_delta!(note_to_period(60, 20, PeriodType::Amiga), 424.154528);
        assert_delta!(note_to_period(60, 20, PeriodType::Linear), 2877.500000);
    }

    #[test]
    fn test_period_to_bend() {
        assert_eq!(period_to_bend(500.0_f64, 0, PeriodType::Amiga), 0);
        assert_eq!(period_to_bend(500.0_f64, 60, PeriodType::Amiga), -34455);
        assert_eq!(period_to_bend(500.0_f64, 60, PeriodType::Linear), 1904000);
    }
}
