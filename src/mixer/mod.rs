use std::ptr;
use module::sample::{Sample, SampleType};
use mixer::interpolator::{Interpolator, Interpolate};
use util;
use ::*;

mod interpolator;

const PAL_RATE     : usize = 250;
const C4_PERIOD    : f64 = 428.0;
const SMIX_SHIFT   : usize = 16;
const SMIX_MASK    : usize = 0xffff;
const LIM16_HI     : i32 = 32767;
const LIM16_LO     : i32 = -32768;
const DOWNMIX_SHIFT: usize = 8;


pub struct Mixer<'a> {

    pub rate  : usize,
    mute      : bool,
    voices    : Vec<Voice>,
    framesize : usize,
    buf32     : [i32; MAX_FRAMESIZE],
    buffer    : [i16; MAX_FRAMESIZE],
    pub interp: interpolator::Interpolator,
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
            interp   : Interpolator::Linear,
            sample,
        }
    }

    pub fn num_voices(&self) -> usize {
        self.voices.len()
    }

    pub fn create_voices(&mut self, num: usize) {
        self.voices = vec![Voice::new(); num];

        for i in 0..self.voices.len() {
            self.voices[i].num = i;
        }
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

    pub fn voicepos(&self, voice: usize) -> f64 {
        if voice >= self.voices.len() {
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
        if voice >= self.voices.len() {
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
        if voice >= self.voices.len() {
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
println!("mixer::set_volume voice={} vol={}", voice, vol);
        if voice >= self.voices.len() {
            return
        }

        self.voices[voice].vol = vol;
    }

    pub fn set_pan(&mut self, voice: usize, pan: isize) {
        if voice >= self.voices.len() {
            return
        }

        self.voices[voice].pan = pan;
    }

    pub fn set_period(&mut self, voice: usize, period: f64) {
println!("mixer::set_period: voice={}/{}, period={}", voice, self.voices.len(), period);
        if voice >= self.voices.len() {
            return
        }

        self.voices[voice].period = period;
    }

    pub fn set_patch(&mut self, voice: usize, ins: usize, smp: usize, ac: bool) {
println!("voice:{} set patch {}", voice, ins);
        if voice >= self.voices.len() {
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

println!("--- mix");

        self.framesize = self.rate * PAL_RATE * 2 / bpm / 100;

        let mut md = MixerData{
            pos    : 0.0_f64,
            buf_pos: 0,
            step   : 0,
            size   : 0,
            vol_r  : 0,
            vol_l  : 0,
        };

        unsafe { ptr::write_bytes(self.buf32.as_mut_ptr(), 0, self.buf32.len() - 1); }

        for mut v in &mut self.voices {
println!("mixer::mix: {:?}", v);
println!("mix voice {}", v.num);
println!("sample = {}, period = {}", v.smp, v.period);
            if v.period < 1.0 {
                continue
            }

            let mut buf_pos = 0;

            let vol_r = v.vol * (0x80 - v.pan) as usize;
            let vol_l = v.vol * (0x80 + v.pan) as usize;
        
            let sample = &self.sample[v.smp];
            let step = C4_PERIOD * sample.rate / self.rate as f64 / v.period;
            if step < 0.001 {
                continue;
            }
println!("step = {}", step);

            let lps = sample.loop_start;
            let lpe = sample.loop_end;

            let mut usmp = 0;
            let mut size = self.framesize as isize;
            loop {
                if size <= 0 {
                    break
                }

                // How many samples we can write before the loop break or sample end...
                let mut samples = 0;
                if v.pos > v.end as f64 {
                    usmp = 1;
                } else {
                    let mut s = ((v.end as f64 - v.pos) / step).ceil() as isize;
                    // ...inside the tick boundaries
                    if s > size {
                       s = size;
                    }
                    samples = s;
                    if samples > 0 {
                        usmp = 0;
                    }
                }
//println!("v.pos={}, v.end={}, samples={}, v.vol={}", v.pos, v.end, samples, v.vol);

                if v.vol > 0 {
                    let mix_size = samples * 2;

                    if samples > 0 {
                        md.pos = v.pos + 2.0;
                        md.buf_pos = buf_pos;
                        md.step = (step * (1_u32 << SMIX_SHIFT) as f64) as usize;
                        md.size = samples;
                        md.vol_l = vol_l >> 8;
                        md.vol_r = vol_r >> 8;

                        match sample.sample_type {
                            SampleType::Empty    => {},
                            SampleType::Sample8  => md.mix::<i8>(&self.interp, &sample.data_8(), &mut self.buf32),
                            SampleType::Sample16 => md.mix::<i16>(&self.interp, &sample.data_16(), &mut self.buf32),
                        };

                        buf_pos += mix_size as usize;
                    }
                }
                v.pos += step * samples as f64 / 2.0;

                // No more samples in this frame
                size -= samples + usmp;
            }
        }

        // Render final frame
        self.downmix();
    }

    fn downmix(&mut self) {

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


#[derive(Clone,Debug,Default)]
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
    pub pos    : f64,
    pub buf_pos: usize,
    pub step   : usize,
    pub size   : isize,
    pub vol_l  : usize,
    pub vol_r  : usize,
}

impl MixerData {
    fn mix<T>(&mut self, interp: &Interpolator, data: &[T], buf32: &mut [i32])
    where interpolator::Nearest: interpolator::Interpolate<T>,
          interpolator::Linear : interpolator::Interpolate<T>
    {
        let mut pos = self.pos as usize;
        let mut frac = ((1 << SMIX_SHIFT) as f64 * (self.pos - pos as f64)) as usize;
        let mut bpos = self.buf_pos;

        for n in 0..self.size {
            let i = &data[pos-1..pos+2];

            let smp = match interp {
                &Interpolator::Nearest => interpolator::Nearest.get_sample(i, frac as i32),
                &Interpolator::Linear  => interpolator::Linear.get_sample(i, frac as i32),
            };

            frac += self.step;
            pos += frac >> SMIX_SHIFT;
            frac &= SMIX_MASK;

            buf32[bpos    ] += smp * self.vol_r as i32;
            buf32[bpos + 1] += smp * self.vol_l as i32;
            bpos += 2;
        }
    }
}
