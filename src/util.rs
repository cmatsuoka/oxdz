use byteorder::{ByteOrder, BigEndian};
use Error;

pub trait BinaryRead {
    fn read_string(&self, ofs: usize, size: usize) -> Result<String, Error>;
    fn read32b(&self, ofs: usize) -> Result<u32, Error>;
    fn read16b(&self, ofs: usize) -> Result<u16, Error>;
    fn read8(&self, ofs: usize) -> Result<u8, Error>;
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
}

fn check_buffer_size(b: &[u8], size: usize) -> Result<(), Error> {
    if size > b.len() {
        return Err(Error::Load("short read"))
    }
    Ok(())
}
