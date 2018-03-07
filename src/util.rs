use std::ptr;
use byteorder::{ByteOrder, BigEndian, LittleEndian};
use Error;
use ::*;

pub const NOTES: &'static [&'static str] = &[
    "C ", "C#", "D ", "D#", "E ", "F ", "F#", "G ", "G#", "A ", "A#", "B "
];


#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { println!("** {}", format!($($arg)*)) })
}

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

#[macro_export]
macro_rules! magic4 {
    ( $a:expr, $b:expr, $c:expr, $d:expr ) =>
        ((($a as u32) << 24) | (($b as u32) << 16) | (($c as u32) << 8) | ($d as u32))
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

