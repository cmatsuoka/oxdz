use module::sample::{Sample, SampleType};
use mixer::interpolator::{AnyInterpolator, Interpolate};
use ::*;

mod interpolator;


pub struct Mixer {

    pub rate  : usize,
    mute      : bool,
    voices    : Vec<Voice>,
    buffer    : [i32; MAX_FRAMESIZE],
    pub interp: interpolator::AnyInterpolator,
}


impl Mixer {

    pub fn new(num: usize) -> Self {
        Mixer {
            rate  : 44100,
            mute  : false,
            voices: vec![Voice::new(); num],
            buffer: [0; MAX_FRAMESIZE],
            interp: AnyInterpolator::Linear(interpolator::Linear),
        }
    }

    pub fn voice_root(&self, voice: usize) -> Option<usize> {
        if voice >= self.voices.len() {
            None
        } else {
            self.voices[voice].root
        }
    }

    pub fn voice_chn(&self, voice: usize) -> Option<usize> {
        if voice >= self.voices.len() {
            None
        } else {
            self.voices[voice].chn
        }
    }

    pub fn reset_voice(&self, voice: usize) {
    }

    pub fn set_volume(&self, voice: usize, vol: usize) {
 
    }

    pub fn set_pan(&self, voice: usize, pan: isize) {
 
    }

    fn mix(&self, samples: &Vec<Sample>) {
        for v in &self.voices {
            let sample = &samples[v.smp];
            match sample.sample_type {
                SampleType::Empty    => {},
                SampleType::Sample8  => self.mix_data::<i8>(&v, &sample.data::<i8>()),
                SampleType::Sample16 => self.mix_data::<i16>(&v, &sample.data::<i16>()),
            };
        }
    }

    fn mix_data<T>(&self, v: &Voice, data: &[T])
    where interpolator::NearestNeighbor: interpolator::Interpolate<T>,
          interpolator::Linear: interpolator::Interpolate<T>
    {
        let p = v.pos as usize;
        let i = &data[p-1..p+2];

        let smp = match &self.interp {
            &AnyInterpolator::NearestNeighbor(ref int) => int.get_sample(i, 0),
            &AnyInterpolator::Linear(ref int)          => int.get_sample(i, 0),
        };

        println!("sample value is {}", smp);
    }
}



#[derive(Clone, Default)]
struct Voice {

    num    : usize,
    root   : Option<usize>,
    chn    : Option<usize>,
    pos    : f64,
    period : f64,
    note   : usize,
    pan    : isize,
    vol    : usize,
    ins    : usize,
    smp    : usize,
}

impl Voice {
    pub fn new() -> Self {
        let v: Voice = Default::default();
        v
    }
}
