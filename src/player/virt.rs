use mixer::Mixer;
use module::Sample;
use ::*;


#[derive(Clone)]
struct VirtChannel {
    count: usize,
    map  : Option<usize>,
}

impl VirtChannel {
    pub fn new() -> Self {
        VirtChannel {
            count: 0,
            map  : None,
        }
    }
}


pub struct Virtual<'a> {
    num_tracks   : usize,              // number of tracks
    virt_numch   : usize,              // number of virtual channels
    virt_used    : usize,              // number of voices currently in use
    virt_channel : Vec<VirtChannel>,
    channel_mute : [bool; MAX_CHANNELS],

    mixer        : Mixer<'a>,
}


impl<'a> Virtual<'a> {
    pub fn new(chn: usize, sample: &'a Vec<Sample>, has_virt: bool) -> Self {

        let mixer = Mixer::new(chn, &sample);
        let num = mixer.num_voices();

        let mut v = Virtual {
            num_tracks  : chn,
            virt_numch  : chn,
            virt_used   : 0,
            virt_channel: Vec::new(),
            channel_mute: [false; MAX_CHANNELS],
            mixer,
        };

        if has_virt {
            v.virt_numch = num;
        }
        v.virt_channel = vec![VirtChannel::new(); v.virt_numch];
        v.mixer.create_voices(chn);
        v
    }

    pub fn root(&self, chn: usize) -> Option<usize> {
        let voice = match self.virt_channel[chn].map {
            Some(val) => val,
            None      => return None,
        };

        self.mixer.voice_root(voice)
    }

    pub fn reset_voice(&mut self, voice: usize, mute: bool) {
        if mute {
            self.mixer.set_volume(voice, 0)
        }

        let root = self.mixer.voice_root(voice).unwrap();
        let chn = self.mixer.voice_chn(voice).unwrap();

        self.virt_used -= 1;
        self.virt_channel[root].count -= 1;
        self.virt_channel[chn].map = None;
        self.mixer.reset_voice(voice);
    }

    pub fn alloc_voice(&mut self, chn: usize) -> usize {
        // Locate free voice
        let num = match self.mixer.find_free_voice() {
            Some(v) => v,
            None    => self.free_voice(),
        };

        self.virt_channel[chn].count += 1;
        self.virt_used += 1;
        self.mixer.set_voice(num, chn);
        self.virt_channel[chn].map = Some(num);

        num
    }

    pub fn free_voice(&mut self) -> usize {

        // Find background voice with lowest volume
        let num = self.mixer.find_lowest_voice(self.num_tracks);

        let root = self.mixer.voice_root(num).unwrap();
        let chn = self.mixer.voice_chn(num).unwrap();
        self.virt_channel[chn].map = None;
        self.virt_channel[root].count -= 1;
        self.virt_used -= 1;

        num
    }

    fn channel_to_voice(&self, chn: usize) -> Option<usize> {
        if chn >= self.virt_numch {
            None
        } else {
            self.virt_channel[chn].map
        }
    }

    pub fn set_volume(&mut self, chn: usize, mut vol: usize) {
        let voice = try_option!(self.channel_to_voice(chn));

        match self.mixer.voice_root(voice) {
            Some(v) => if self.channel_mute[v] { vol = 0 },
            None    => vol = 0,
        }

        self.mixer.set_volume(voice, vol);

        // reset voice if volume is 0 on a virtual channel
        if vol == 0 && chn >= self.num_tracks {
            self.reset_voice(voice, true)
        }
    }

    pub fn set_pan(&mut self, chn: usize, pan: isize) {
        let voice = try_option!(self.channel_to_voice(chn));
        self.mixer.set_pan(voice, pan);
    }

    pub fn set_period(&mut self, chn: usize, period: f64) {
        let voice = try_option!(self.channel_to_voice(chn));
        self.mixer.set_period(voice, period);
    }

    pub fn set_voicepos(&mut self, chn: usize, pos: f64) {
        let voice = try_option!(self.channel_to_voice(chn));
        self.mixer.set_voicepos(voice, pos, true);
    }

    pub fn set_patch(&mut self, chn: usize, ins: usize, smp: usize, note: usize) {

        let voice = match self.channel_to_voice(chn) {
            Some(v) => v,  // TODO: act stuff
            None    => self.alloc_voice(chn),
        };

        self.mixer.set_patch(voice, ins, smp, true);
        self.mixer.set_note(voice, note);
    }

    pub fn mix(&mut self, bpm: usize) {
        self.mixer.mix(bpm)
    }

    pub fn buffer(&self) -> &[i16] {
        self.mixer.buffer()
    }
}

