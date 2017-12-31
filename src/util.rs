use byteorder::{ByteOrder, BigEndian};

pub trait BinaryRead {
    fn read_string(&self, ofs: usize, size: usize) -> String;
    fn read32b(&self, ofs: usize) -> u32;
    fn read16b(&self, ofs: usize) -> u16;
}

impl<'a> BinaryRead for &'a [u8] {
    fn read_string(&self, ofs: usize, size: usize) -> String {
        String::from_utf8_lossy(&self[ofs..ofs+size]).to_string()
    }

    fn read32b(&self, ofs: usize) -> u32 {
        BigEndian::read_u32(&self[ofs..ofs+4])
    }

    fn read16b(&self, ofs: usize) -> u16 {
        BigEndian::read_u16(&self[ofs..ofs+2])
    }
}
