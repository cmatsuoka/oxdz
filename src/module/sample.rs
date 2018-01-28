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
    pub size        : usize,
    pub loop_start  : usize,
    pub loop_end    : usize,
    pub sloop_start : usize,
    pub sloop_end   : usize,
    pub has_loop    : bool,
    pub has_sloop   : bool,
    pub loop_bidir  : bool,
    pub loop_full   : bool,
    pub sloop_bidir : bool,
    pub guard_size  : usize,
    pub rate        : f64,
    pub name        : String,
    data            : Vec<u8>,
}

impl Sample {
    pub fn new() -> Sample {
        Sample {
            sample_type : SampleType::Empty,
            num         : 0,
            size        : 0,
            loop_start  : 0,
            loop_end    : 0,
            sloop_start : 0,
            sloop_end   : 0,
            has_loop    : false,
            has_sloop   : false,
            loop_bidir  : false,
            loop_full   : false,
            sloop_bidir : false,
            guard_size  : 0,
            rate        : 8000_f64,
            name        : "".to_owned(),
            data        : vec![0; GUARD_SIZE],  // start guard bytes
        }
    }

    pub fn store(&mut self, b: &[u8]) {
        self.data.extend(b);
        self.data.extend([0; GUARD_SIZE].iter());  // add end guard bytes
    }

    pub fn data_8(&self) -> &[i8] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr().offset(2) as *const i8, self.size + 2 * (GUARD_SIZE/2) as usize)
        }
    }

    pub fn data_16(&self) -> &[i16] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const i16, self.size + 2 * GUARD_SIZE as usize)
        }
    }

    pub fn sanity_check(&mut self) {
        if self.loop_start > self.size {
            self.has_loop = false;
        }
        if self.loop_end > self.size {
            self.loop_end = self.size;
        }
        if self.loop_end <= self.loop_start {
            self.has_loop = false;
        }
    }

    pub fn to_signed(&mut self) {
        match &self.sample_type {
            Sample8  => {
                for i in 2..self.size+2 {
                    self.data[i] = self.data[i].wrapping_add(0x80);
                }
            },
            _ => ()
        }
    }
}

