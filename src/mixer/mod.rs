use module::sample::{Sample, SampleType};
use mixer::interpolator::{AnyInterpolator, Interpolate};
use util;
use ::*;

mod interpolator;

const C4_PERIOD    : f64 = 428.0;
const SMIX_SHIFT   : usize = 16;
const SMIX_MASK    : usize = 0xffff;
const LIM16_HI     : i32 = 32767;
const LIM16_LO     : i32 = -32768;
const DOWNMIX_SHIFT: usize = 10;


pub struct Mixer<'a> {

    pub rate  : usize,
    mute      : bool,
    voices    : Vec<Voice>,
    framesize : usize,
    buf32     : [i32; MAX_FRAMESIZE],
    buffer    : [i16; MAX_FRAMESIZE],
    pub interp: interpolator::AnyInterpolator,
    sample    : &'a Vec<Sample>,
}


impl<'a> Mixer<'a> {

    pub fn new(num: usize, sample: &'a Vec<Sample>) -> Self {
        Mixer {
            rate     : 44100,
            mute     : false,
            voices   : Vec::new(),
            framesize: 0,
            buf32    : [0; MAX_FRAMESIZE],
            buffer   : [0; MAX_FRAMESIZE],
            interp   : AnyInterpolator::Linear(interpolator::Linear),
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

    pub fn set_patch(&mut self, voice: usize, ins: usize, smp: usize, ac: bool) {
println!("voice:{} set patch {}", voice, ins);
        if voice < self.voices.len() {
            return
        }

        self.set_voicepos(voice, 0.0, ac);

        let v = &mut self.voices[voice];
        v.ins = ins;
        v.smp = smp;
        v.vol = 0;
        v.pan = 0; 
        v.has_loop = false;

        let sample = &self.sample[v.smp];

        v.pos = 0_f64;
        v.end = sample.size;
        
        // ...

    }

    pub fn mix(&mut self, bpm: usize) {

println!("mix");

        self.framesize = self.rate * PAL_RATE / bpm / 100;

        let mut md = MixerData{
            pos    : 0,
            buf_pos: 0,
            step   : 0,
            size   : 0,
        };

        for mut v in &mut self.voices {
println!("mix voice");
println!("sample = {}", v.smp);
            if v.period < 1.0 {
                continue
            }
        
            let sample = &self.sample[v.smp];
            let step = C4_PERIOD * sample.rate / self.rate as f64 / v.period;
println!("rate = {}", self.rate);
//println!("sample = {:?}", sample);

println!("step = {}", step);
            let mut size = self.framesize;
            loop {
                if size <= 0 {
                    break
                }

                let mut buf_pos = 0;
                let mut samples = 0;

                // How many samples we can write before the loop break or sample end...
                if v.pos < v.end as f64 {
                    let mut s = ((v.end as f64 - v.pos) / step).ceil() as usize;
                    // ...inside the tick boundaries
                    if s > self.framesize {
                       s = self.framesize;
                    }
                    samples = s;
                }
println!("v.pos={}, v.end={}, samples={}", v.pos, v.end, samples);

                if v.vol > 0 {
                    let mix_size = samples * 2;

                    if samples > 0 {

                        md.pos = v.pos as usize;
                        md.buf_pos = buf_pos;
                        md.step = (step * (1_u32 << SMIX_SHIFT) as f64) as usize;
                        md.size = samples;

                        match sample.sample_type {
                            SampleType::Empty    => {},
                            SampleType::Sample8  => md.mix_data::<i8>(&self.interp, &sample.data::<i8>(), &mut self.buf32),
                            SampleType::Sample16 => md.mix_data::<i16>(&self.interp, &sample.data::<i16>(), &mut self.buf32),
                        };

                        buf_pos += mix_size;
                    }
                }
                v.pos += step * samples as f64;
                size -= samples;
                // TODO: handle loop
            }
        }

        // Render final frame
        self.downmix();
    }

    fn downmix(&mut self) {
        println!("downmix");

        let mut i = 0;
        loop {
            if i >= self.framesize {
                break;
            }

            let smp = self.buf32[i] >> DOWNMIX_SHIFT;
            if smp > LIM16_HI {
                self.buffer[i] = LIM16_HI as i16;
            } else if smp < LIM16_LO {
                self.buffer[i] = LIM16_LO as i16;
            } else {
                self.buffer[i] = smp as i16;
            }

            i += 1;
        }
    }

    pub fn buffer(&self) -> &[i16] {
        &self.buffer[..self.framesize]
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


struct MixerData {
    pub pos: usize,
    pub buf_pos: usize,
    pub step: usize,
    pub size: usize
}

impl MixerData {
    fn mix_data<T>(&mut self, interp: &AnyInterpolator, data: &[T], buf32: &mut [i32])
    where interpolator::NearestNeighbor: interpolator::Interpolate<T>,
          interpolator::Linear: interpolator::Interpolate<T>
    {
        println!("mix_data");

        for n in 0..self.size {
            let i = &data[self.pos-1..self.pos+2];

            let smp = match interp {
                &AnyInterpolator::NearestNeighbor(ref int) => int.get_sample(i, 0),
                &AnyInterpolator::Linear(ref int)          => int.get_sample(i, 0),
            };

            self.pos += self.step;

            buf32[self.buf_pos] = smp;
            println!("sample: {}", smp);
        }
    }
}
