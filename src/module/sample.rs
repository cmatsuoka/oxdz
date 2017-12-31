use std::slice;

#[derive(Debug)]
pub enum SampleType {
    Sample8,
    Sample16,
    Empty,
}

#[derive(Debug)]
pub struct Sample {
    pub sample_type : SampleType,
    pub length      : u32,
    pub loop_start  : u32,
    pub loop_end    : u32,
    pub sloop_start : u32,
    pub sloop_end   : u32,
    pub has_loop    : bool,
    pub has_sloop   : bool,
    pub loop_bidir  : bool,
    pub loop_full   : bool,
    pub sloop_bidir : bool,
    pub guard_size  : u32,
    pub rate        : f64,
    pub name        : String,
    data            : Vec<u8>,
}

impl Sample {
    pub fn new() -> Sample {
        Sample {
            sample_type : SampleType::Empty,
            length      : 0,
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
            data        : Vec::new(),
        }
    }

    pub fn data<T>(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.data.as_ptr() as *const T, self.length as usize)
        }
    }
}

