use std::slice;

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
            data        : vec![0; GUARD_SIZE],
        }
    }

    pub fn store(&mut self, b: &[u8]) {
        self.data.extend(b);
        self.data.extend([0; GUARD_SIZE].iter());  // add guard bytes
    }

    pub fn data<T>(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const T, self.size as usize)
        }
    }
}

