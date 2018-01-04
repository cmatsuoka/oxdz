use mixer::Mixer;


struct VirtChannel {
    count: usize,
    map  : Option<usize>,
}


pub struct Virtual {
    num_tracks   : usize,             // number of tracks
    virt_channels: usize,             // number of virtual channels
    virt_used    : usize,             // number of voices currently in use
    virt_limit   : usize,             // number of sound card voices
    virt_channel : Vec<VirtChannel>,

    mixer        : Mixer,
}

impl Virtual {
    pub fn new(mixer: Mixer) -> Self {
        Virtual {
            num_tracks   : 0,
            virt_channels: 0,
            virt_used    : 0,
            virt_limit   : 0,
            virt_channel : Vec::new(),
            mixer,
        }
    }

    pub fn root(&self, chn: usize) -> Option<usize> {
        let voice = match self.virt_channel[chn].map {
            Some(val) => val,
            None      => return None,
        };

        self.mixer.voice_root(voice)
    }

    pub fn reset_voice(&mut self, voice: usize, mute: bool) {
        if voice >= self.virt_limit {
            return
        }

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

    pub fn channel_to_voice(&self, chn: usize) -> Option<usize> {
        if chn >= self.virt_channels {
            None
        } else {
            self.virt_channel[chn].map
        }
    }

    pub fn set_volume(&mut self, chn: usize, vol: usize) {
        let voice = match self.channel_to_voice(chn) {
            Some(v) => v,
            None    => return,
        };

        /* TODO: check if root is muted */

        self.mixer.set_volume(voice, vol);

        // reset voice if volume is 0 on a virtual channel
        if vol == 0 && chn >= self.num_tracks {
            self.reset_voice(voice, true)
        }
    }

    pub fn set_pan(&self, chn: usize, pan: isize) {
        let voice = match self.channel_to_voice(chn) {
            Some(v) => v,
            None    => return,
        };

        self.mixer.set_pan(voice, pan);
    }
}

