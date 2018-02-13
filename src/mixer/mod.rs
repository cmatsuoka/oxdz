use module::sample::{Sample, SampleType};
use mixer::interpolator::{Interpolator, Interpolate};
use mixer::paula::Paula;
use util::MemOpExt;
use util;
use ::*;

mod interpolator;
mod paula;

const PAL_RATE     : f64 = 250.0;
const C4_PERIOD    : f64 = 428.0;
const SMIX_SHIFT   : usize = 16;
const SMIX_MASK    : usize = 0xffff;
const LIM16_HI     : i32 = 32767;
const LIM16_LO     : i32 = -32768;
const DOWNMIX_SHIFT: usize = 12;

macro_rules! try_voice {
    ( $a:expr, $b: expr ) => {
        if $a >= $b.len() {
            return
        }
    };
    ( $a:expr, $b:expr, $c:expr ) => {
        if $a >= $b.len() {
            return $c
        }
    };
}


pub struct Mixer<'a> {
    pub rate  : u32,
    pub factor: f64,  // tempo factor multiplier
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
        let mut mixer = Mixer {
            rate     : 44100,
            factor   : 1.0,
            mute     : false,
            voices   : vec![Voice::new(); num],
            framesize: 0,
            buf32    : [0; MAX_FRAMESIZE],
            buffer   : [0; MAX_FRAMESIZE],
            interp   : Interpolator::Spline,
            sample,
        };

        for i in 0..num {
            mixer.voices[i].num = i;
        }

        mixer
    }

    pub fn num_voices(&self) -> usize {
        self.voices.len()
    }

    pub fn enable_paula(&mut self, enable: bool) {
        for v in &mut self.voices {
            v.paula = if enable {
                Some(Paula::new(self.rate))
            } else {
                None
            }
        }
    }

    pub fn set_tempo(&mut self, tempo: usize) {
        self.framesize = ((self.rate as f64 * PAL_RATE) / (self.factor * tempo as f64 * 100.0)) as usize;
    }

    pub fn reset_voice(&self, voice: usize) {
    }

    pub fn voicepos(&self, voice: usize) -> f64 {
        try_voice!(voice, self.voices, 0_f64);

        let v = &self.voices[voice];

/*
        let sample = &self.sample[v.smp];
        if v.has_loop && sample.loop_bidir {
            // TODO: handle bidirectional loop
        }
*/
        
        v.pos
    }

    pub fn set_voicepos(&mut self, voice: usize, pos: f64) {
        try_voice!(voice, self.voices);

        let v = &mut self.voices[voice];
        v.pos = pos;

        let sample = &self.sample[v.smp];

        v.adjust_end(&sample);

        if v.pos >= v.end as f64 {
            if v.has_loop {
                v.pos = v.loop_start as f64;
            } else {
                v.pos = sample.size as f64;
            }
        }

        // TODO: handle bidirectional loop

        //if ac {
        //    v.anticlick();
        //}
    }

    pub fn set_note(&mut self, voice: usize, mut note: usize) {
        try_voice!(voice, self.voices);

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
        try_voice!(voice, self.voices);
        self.voices[voice].vol = vol;
    }

    pub fn set_pan(&mut self, voice: usize, pan: isize) {
        try_voice!(voice, self.voices);
        self.voices[voice].pan = pan;
    }

    pub fn set_period(&mut self, voice: usize, period: f64) {
        try_voice!(voice, self.voices);
        self.voices[voice].period = period;
    }

    pub fn set_sample(&mut self, voice: usize, smp: usize) {
        try_voice!(voice, self.voices);

        if smp == 0 {
            return
        }

        let v = &mut self.voices[voice];
        v.smp = smp - 1;
        v.pos = 0_f64;
        v.end = self.sample[smp].size;
        v.has_loop = false;
        v.sample_end = true;
    }

    pub fn set_sample_ptr(&mut self, voice: usize, addr: u32) {
        try_voice!(voice, self.voices);

        let v = &mut self.voices[voice];

        for s in self.sample {
            if addr >= s.address && addr < s.address + s.size {
                v.smp = s.num - 1;
                v.pos = (addr - s.address) as f64;
                v.end = s.size;
                break
            }
        }
    }

    pub fn set_loop_start(&mut self, voice: usize, val: u32) {
        try_voice!(voice, self.voices);
        self.voices[voice].loop_start = val;
    }

    pub fn set_loop_end(&mut self, voice: usize, val: u32) {
        try_voice!(voice, self.voices);
        self.voices[voice].loop_end = val;
    }

    pub fn enable_loop(&mut self, voice: usize, val: bool) {
        try_voice!(voice, self.voices);
        self.voices[voice].has_loop = val;
    }

    pub fn mix(&mut self) {

        let mut md = MixerData{
            pos    : 0.0_f64,
            buf_pos: 0,
            step   : 0,
            size   : 0,
            vol_r  : 0,
            vol_l  : 0,
        };

        self.buf32[..].fill(0, self.framesize);

        for v in &mut self.voices {
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

            //let lps = sample.loop_start;
            //let lpe = sample.loop_end;

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

                if v.vol > 0 {
                    let mix_size = samples * 2;

                    if samples > 0 {
                        md.pos = v.pos;
                        md.buf_pos = buf_pos;
                        md.step = (step * (1_u32 << SMIX_SHIFT) as f64) as usize;
                        md.size = samples;
                        md.vol_l = vol_l >> 8;
                        md.vol_r = vol_r >> 8;

                        match v.paula {
                            Some(ref mut val) => md.mix_paula(&sample.data_8(), &mut self.buf32, val),
                            None          => {
                                match sample.sample_type {
                                    SampleType::Empty    => {},
                                    SampleType::Sample8  => md.mix::<i8>(&self.interp, &sample.data_8(), &mut self.buf32, &mut v.i_buffer),
                                    SampleType::Sample16 => md.mix::<i16>(&self.interp, &sample.data_16(), &mut self.buf32, &mut v.i_buffer),
                                };
                            }
                        }

                        buf_pos += mix_size as usize;
                    }
                }
                v.pos += step * samples as f64;
                size -= samples + usmp;

                // No more samples in this frame
                if size <= 0 {
                    if v.has_loop {
                        if v.pos + step >= v.end as f64 {
                            v.pos += step;
                            v.loop_reposition(&sample);
                        }
                    }
                    continue;
                }

                // First sample loop run
                if !v.has_loop {
                    v.sample_end = true;
                    size = 0;
                    continue;
                }

                v.loop_reposition(&sample);
            }
        }

        // Render final frame
        self.downmix();
    }


    fn downmix(&mut self) {

        let size = self.framesize * 2;
        let mut i = 0;
        loop {
            if i >= size {
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
        // *2 because we're stereo
        &self.buffer[..self.framesize*2]
    }
}


#[derive(Clone,Default)]
struct Voice {
    num       : usize,
    pos       : f64,
    period    : f64,
    note      : usize,
    pan       : isize,
    vol       : usize,
    ins       : usize,
    smp       : usize,
    end       : u32,
    loop_start: u32,
    loop_end  : u32,
    has_loop  : bool,
    sample_end: bool,

    i_buffer  : [i32; 4],

    paula     : Option<Paula>,
}

impl Voice {
    pub fn new() -> Self {
        let v: Voice = Default::default();
        v
    }

    pub fn adjust_end(&mut self, sample: &Sample) {
        // sanity check
        if self.loop_end > sample.size {
            self.loop_end = sample.size;
        }

        if self.has_loop {
            /*if self.loop_full && !self.has_loop {
                self.end = sample.size;
            } else*/ {
                self.end = self.loop_end;
            }
        } else {
            self.end = sample.size;
        }
    }

    pub fn loop_reposition(&mut self, sample: &Sample) {
        // sanity check
        if self.loop_start > sample.size {
            self.has_loop = false;
            return
        }
        if self.loop_end > sample.size {
            self.loop_end = sample.size;
        }
        
        let loop_size = self.loop_end - self.loop_start;

        // Reposition for next loop
        self.pos -= loop_size as f64;  // forward loop
        self.end = self.loop_end;
        self.has_loop = true;

        //if self.bidir_loop {
        //}
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
    fn mix<T>(&mut self, interp: &Interpolator, data: &[T], buf32: &mut [i32], ibuf: &mut [i32])
    where Sampler: SamplerOperations<T>
    {
        let mut pos = self.pos as usize;
        let mut frac = ((1 << SMIX_SHIFT) as f64 * (self.pos - pos as f64)) as usize;
        let mut bpos = self.buf_pos;

        let bmax = match interp {
            &Interpolator::Nearest => interpolator::Nearest::bsize(),
            &Interpolator::Linear  => interpolator::Linear::bsize(),
            &Interpolator::Spline  => interpolator::Spline::bsize(),
        } - 1;

        for _ in 0..self.size {
            frac += self.step;
            let istep = frac >> SMIX_SHIFT;
            frac &= SMIX_MASK;

            // add sample to interpolation buffer
            if istep > 0 {
                for i in 0..bmax {
                    ibuf[i] = ibuf[i+1]
                }
                ibuf[bmax] = Sampler::get(&data[pos]);
                pos += istep;
            }

            let smp = match interp {
                &Interpolator::Nearest => interpolator::Nearest.get_sample(&ibuf, frac as i32),
                &Interpolator::Linear  => interpolator::Linear.get_sample(&ibuf, frac as i32),
                &Interpolator::Spline  => interpolator::Spline.get_sample(&ibuf, frac as i32),
            };

            // Store stereo
            buf32[bpos    ] += smp * self.vol_r as i32;
            buf32[bpos + 1] += smp * self.vol_l as i32;
            bpos += 2;

        }
    }

    fn mix_paula(&self, data: &[i8], buf32: &mut [i32], paula: &mut Paula) {
        let mut pos = self.pos as usize;
        let mut frac = ((1 << SMIX_SHIFT) as f64 * (self.pos - pos as f64)) as usize;
        let mut bpos = self.buf_pos;
        let filter = 0;

        for _ in 0..self.size {
            let num_in = paula.remainder as usize / paula::MINIMUM_INTERVAL;
            let ministep = self.step / num_in;

            // input is sampled at a higher rate than output
            for _ in 0..num_in-1 {
                paula.input_sample(*&data[pos] as i16);
                paula.do_clock(paula::MINIMUM_INTERVAL as i16);

                frac += ministep;
                pos += frac >> SMIX_SHIFT;
                frac &= SMIX_MASK;
            }

            paula.input_sample(*&data[pos] as i16);

            paula.remainder -= (num_in * paula::MINIMUM_INTERVAL) as f64;
            let remainder = paula.remainder as i16;
            paula.do_clock(remainder);

            frac += self.step - (num_in-1)*ministep;
            pos += frac >> SMIX_SHIFT;
            frac &= SMIX_MASK;

            let smp = paula.output_sample(filter) as i32;
            let cycles = (paula::MINIMUM_INTERVAL - paula.remainder as usize) as i16;
            paula.do_clock(cycles);

            paula.remainder += paula.fdiv;

            // Store stereo
            buf32[bpos    ] += smp * (self.vol_r << 8) as i32;
            buf32[bpos + 1] += smp * (self.vol_l << 8) as i32;
            bpos += 2;
        }
    }
}


struct Sampler;

trait SamplerOperations<T> {
    fn get(&T) -> i32;
}

impl SamplerOperations<i16> for Sampler {
    fn get(i: &i16) -> i32 {
        *i as i32
    }
}

impl SamplerOperations<i8> for Sampler {
    fn get(i: &i8) -> i32 {
        (*i as i32) << 8
    }
}

