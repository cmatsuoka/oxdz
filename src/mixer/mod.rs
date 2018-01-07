use module::sample::{Sample, SampleType};
use mixer::interpolator::{AnyInterpolator, Interpolate};
use util;
use ::*;

mod interpolator;


pub struct Mixer<'a> {

    pub rate    : usize,
    mute        : bool,
    voices      : Vec<Voice>,
    buffer      : [i32; MAX_FRAMESIZE],
    pub interp  : interpolator::AnyInterpolator,
    sample      : &'a Vec<Sample>,
}


impl<'a> Mixer<'a> {

    pub fn new(num: usize, sample: &'a Vec<Sample>) -> Self {
        Mixer {
            rate  : 44100,
            mute  : false,
            voices: Vec::new(),
            buffer: [0; MAX_FRAMESIZE],
            interp: AnyInterpolator::Linear(interpolator::Linear),
            sample,
        }
    }

    pub fn num_voices(&self) -> usize {
        self.voices.len()
    }

    pub fn create_voices(&mut self, num: usize) {
        self.voices = vec![Voice::new(); num];
    }

    pub fn find_free_voice(&self) -> Option<usize> {
        for (i, v) in self.voices.iter().enumerate() {
            if v.chn == None {
                return Some(i);
            }
        }

        return None;
    }

    pub fn find_lowest_voice(&self, num_tracks: usize) -> usize {
        let mut vol = std::usize::MAX;
        let mut num = 0;

        for (i, v) in self.voices.iter().enumerate() {

            let chn = match v.chn {
                Some(v) => v,
                None    => continue,
            };

            if chn >= num_tracks {   // only background channels
                if v.vol < vol {
                    vol = v.vol;
                    num = i;
                }
            }
        }

        num
    }

    pub fn set_voice(&mut self, num: usize, chn: usize) {
        if num < self.voices.len() {
            self.voices[num].chn = Some(chn);
            self.voices[num].root = Some(chn);
        }
    }

    pub fn voice_root(&self, voice: usize) -> Option<usize> {
        if voice < self.voices.len() {
            self.voices[voice].root
        } else {
            None
        }
    }

    pub fn voice_chn(&self, voice: usize) -> Option<usize> {
        if voice < self.voices.len() {
            self.voices[voice].chn
        } else {
            None
        }
    }

    pub fn reset_voice(&self, voice: usize) {
    }

    pub fn voicepos(&self, voice: usize) -> f64 {
        if voice < self.voices.len() {
            return 0_f64
        }

        let v = &self.voices[voice];
        let sample = &self.sample[v.smp];

        if sample.has_loop && sample.loop_bidir {
            // TODO: handle bidirectional loop
        }
        
        v.pos
    }

    pub fn set_voicepos(&mut self, voice: usize, pos: f64, ac: bool) {
        if voice < self.voices.len() {
            return
        }

        let v = &mut self.voices[voice];
        v.pos = pos;

        let sample = &self.sample[v.smp];

        v.adjust_end(&sample);

        if v.pos >= v.end as f64 {
            if sample.has_loop {
                v.pos = sample.loop_start as f64;
            } else {
                v.pos = sample.size as f64;
            }
        }

        // TODO: handle bidirectional loop

        if ac {
            v.anticlick();
        }
    }

    pub fn set_note(&mut self, voice: usize, mut note: usize) {
        if voice < self.voices.len() {
            return
        }

        // FIXME: Workaround for crash on notes that are too high
        //        see 6nations.it (+114 transposition on instrument 16)
        //
        if note > 149 {
            note = 149;
        }
        self.voices[voice].note = note;
        self.voices[voice].period = util::note_to_period_mix(note, 0);
    }

    pub fn set_volume(&mut self, voice: usize, vol: usize) {
        if voice < self.voices.len() {
            return
        }

        self.voices[voice].vol = vol;
    }

    pub fn set_pan(&mut self, voice: usize, pan: isize) {
        if voice < self.voices.len() {
            return
        }

        self.voices[voice].pan = pan;
    }

    pub fn set_period(&mut self, voice: usize, period: f64) {
        if voice < self.voices.len() {
            return
        }

        self.voices[voice].period = period;
    }

    pub fn set_patch(&mut self, voice: usize, smp: usize, ac: bool) {
        if voice < self.voices.len() {
            return
        }

        self.set_voicepos(voice, 0.0, ac);

        let v = &mut self.voices[voice];
        v.smp = smp;
        v.vol = 0;
        v.pan = 0; 
        v.has_loop = false;

        let sample = &self.sample[v.smp];

        // ...

    }

    fn mix(&self) {
        for v in &self.voices {
            let sample = &self.sample[v.smp];
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
    num     : usize,
    root    : Option<usize>,
    chn     : Option<usize>,
    pos     : f64,
    period  : f64,
    note    : usize,
    pan     : isize,
    vol     : usize,
    ins     : usize,
    smp     : usize,
    end     : usize,
    has_loop: bool,
}

impl Voice {
    pub fn new() -> Self {
        let v: Voice = Default::default();
        v
    }

    pub fn adjust_end(&mut self, sample: &Sample) {
        if sample.has_loop {
            if sample.loop_full && !self.has_loop {
                self.end = sample.size;
            } else {
                self.end = sample.loop_end;
            }
        } else {
            self.end = sample.size;
        }
    }

    pub fn anticlick(&self) {
    }
}
