use module::sample::{Sample, SampleType};
use mixer::interpolator::{AnyInterpolator, Interpolate};
use ::*;

mod interpolator;


pub struct Mixer<'a> {

    rate   : f64,
    mute   : bool,
    voices : Vec<Voice>,
    buffer : [i32; MAX_FRAMESIZE],
    samples: &'a Vec<Sample>,
    interp : interpolator::AnyInterpolator,
}


impl<'a> Mixer<'a> {

    fn mix(&self) {
        for v in &self.voices {
            let sample = &self.samples[v.smp];
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



struct Voice {

    num    : u8,
    root   : u8,
    pos    : f64,
    period : f64,
    note   : u8,
    pan    : i8,
    vol    : u8,
    ins    : usize,
    smp    : usize,
}


