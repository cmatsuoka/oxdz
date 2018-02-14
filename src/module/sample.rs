use std::slice;

// Allow 2 samples in 16-bit
pub const GUARD_SIZE: usize = 4;


#[derive(Debug)]
pub enum SampleType {
    Sample8,
    Sample16,
    Empty,
}

#[derive(Debug)]
pub struct Sample {
    pub sample_type : SampleType,
    pub num         : usize,
    pub address     : u32,
    pub size        : u32,
    pub rate        : f64,
    pub name        : String,
    data            : Vec<u8>,
}

impl Sample {
    pub fn new() -> Sample {
        Sample {
            sample_type : SampleType::Empty,
            num         : 0,
            address     : 0,
            size        : 0,
            rate        : 8000_f64,
            name        : "".to_owned(),
            data        : Vec::new(),
        }
    }

    pub fn store(&mut self, b: &[u8]) {
        self.data.extend(b);
        self.data.extend([0; 2].iter());
        let i = self.data.len();
        if i >= 3 {     // FIXME: workaround for Amiga blep ministeps
            self.data[i-2] = self.data[i-3];
            self.data[i-1] = self.data[i-3];
        }
    }

    pub fn data_8(&self) -> &[i8] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const i8, self.size as usize + 2)
        }
    }

    pub fn data_16(&self) -> &[i16] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const i16, self.size as usize)
        }
    }

    fn data_u16_mut(&self) -> &mut [u16] {
        unsafe {
            slice::from_raw_parts_mut(self.data.as_ptr() as *mut u16, self.size as usize)
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
                let data = self.data_u16_mut();
                for i in 0..self.size as usize {
                    data[i] = data[i].wrapping_add(0x8000);
                }
            }
            _ => ()
        }
    }
}

