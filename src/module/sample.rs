use std::slice;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct SampleData {
    raw: Vec<u8>
}

impl<'a> SampleData {
    pub fn new() -> Self {
        SampleData {
            raw: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn as_slice_u8(&'a self) -> &'a [u8] {
        &self.raw
    }

    pub fn as_slice_i8(&'a self) -> &'a [i8] {
        unsafe {
            slice::from_raw_parts(self.raw.as_ptr() as *const i8, self.raw.len() as usize)
        }
    }

    pub fn as_slice_u16(&'a self) -> &'a [u16] {
        unsafe {
            slice::from_raw_parts(self.raw.as_ptr() as *const u16, self.raw.len() as usize / 2)
        }
    }

    pub fn as_slice_i16(&'a self) -> &'a [i16] {
        unsafe {
            slice::from_raw_parts(self.raw.as_ptr() as *const i16, self.raw.len() as usize / 2)
        }
    }

    pub fn as_slice_u16_mut(&'a mut self) -> &'a mut [u16] {
        unsafe {
            slice::from_raw_parts_mut(self.raw.as_ptr() as *mut u16, self.raw.len() as usize / 2)
        }
    }

}

impl Index<usize> for SampleData {
    type Output = u8;

    fn index(&self, i: usize) -> &u8 {
        &self.raw[i]
    }
}

impl IndexMut<usize> for SampleData {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut u8 {
        &mut self.raw[i]
    }
}


#[derive(Debug, Clone)]
pub enum SampleType {
    Sample8,
    Sample16,
    Empty,
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub sample_type : SampleType,
    pub num         : usize,
    pub address     : u32,
    pub size        : u32,
    pub rate        : f64,
    /// The normalized rate used to play this sample.
    pub name        : String,
    /// The raw PCM-encoded sample data.
    pub data        : SampleData,
}

impl Sample {
    pub fn new() -> Sample {
        Sample {
            sample_type : SampleType::Empty,
            num         : 0,
            address     : 0,
            size        : 0,
            rate        : 1.0,
            name        : "".to_owned(),
            data        : SampleData::new(),
        }
    }

    pub fn store(&mut self, b: &[u8]) {
        self.data.raw.extend(b);
        self.data.raw.extend([0; 2].iter());
        let i = self.data.len();
        if i >= 3 {     // FIXME: workaround for Amiga blep ministeps
            self.data[i-2] = self.data[i-3];
            self.data[i-1] = self.data[i-3];
        }
    }

    pub fn to_signed(&mut self) {
        match self.sample_type {
            SampleType::Sample8  => {
                for i in 0..self.size as usize {
                    self.data[i] = self.data[i].wrapping_add(0x80);
                }
            },
            SampleType::Sample16 => {
                let data = self.data.as_slice_u16_mut();
                for i in 0..self.size as usize {
                    data[i] = data[i].wrapping_add(0x8000);
                }
            }
            _ => ()
        }
    }
}

