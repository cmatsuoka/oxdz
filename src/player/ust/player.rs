use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer};
use format::m15::M15Data;
use mixer::Mixer;

/// Ultimate Soundtracker V27 replayer
///
/// An oxdz player based on the Ultimate Soundtracker V27, written 1987/1988 by
/// Karsten Obarski. "All bugs removed".
///
///         "Just look at it -- so small, innocent and cute. :)"
///                                   -- Olav "8bitbubsy" Sørensen

pub struct ModPlayer {
    options   : Options,

    datachn   : [4; DataChnx],
    lev6save  : u32,
    trkpos    : u32,
    patpos    : u32,
    numpat    : u16,
    enbits    : u16,
    timpos    : u16,
}

impl ModPlayer {
    pub fn new(module: &Module, options: Options) -> Self {
        ModPlayer {
            datachn: vec![DataChnx::new(); 4],
            options,

            mt_speed  : 6,
            mt_songpos: 0,
            mt_pattpos: 0,
            timpos: 0,
            mt_break  : false,
        }
    }

    //------------------------------------------------
    // replay-routine
    //------------------------------------------------

    fn replay_muzak(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        self.timpos += 1;
        if self.timpos == 6 {
            self.replaystep()
        } else {
            //------------------------------------------------
            // time left to handle effects between steps
            //------------------------------------------------

            // chaneleffects
            for i in 0..4 {
                if datachn[i].n_3_effect_number != 0 {
                    self.ceff5(i)
                }
            }
        }
    }

    fn ceff5(&mut self, chn: usize) {
        match self.datachn[chn].n_2_sound_number & 0x0f {
            1 => self.arpreggiato(chn),
            2 => self.pitchbend(chm),
        }
    }

    //------------------------------------------------
    // effect 1 arpreggiato
    //------------------------------------------------

    fn arpreggiato(&mut self, chn: usize, mixer: &mut Mixer) {  // ** spread by time
        let datachn = &mut self.datachn[chn];
        let val = match self.timpos {  // ** get higher note-values or play original
            1 => datachn.n_3_effect_number >> 4    // arp1
            2 => datachn.n_3_effect_number & 0x0f  // arp2
            3 => 0                                 // arp3
            4 => datachn.n_3_effect_number >> 4    // arp1
            5 => datachn.n_3_effect_number & 0x0f  // arp2
            _ => 0
        } as usize;

        // arp4
        mixer.set_period(chn, NOTETABLE[self.n_12_last_saved_note + val] as f64);  // move.w  d2,6(a5)
    }

    //------------------------------------------------
    // effect 2 pitchbend
    //------------------------------------------------

    fn pitchbend(&mut self, chn: usize, mixer: &mut Mixer) {
        let datachn = &mut self.datachn[chn];
        let val = datachn.n_3_effect_number >> 4;
        if val != 0 {
            datach.n_0_current_note += val;                         // add.w   d0,(a6)
            mixer.set_period(chn, datach.n_0_current_note as f64);  // move.w  (a6),6(a5)
            return
        }
        // pit2
        let val = datachn.n_3_effect_number & 0x0f;
        if val != 0 {
            datach.n_0_current_note -= val;                         // sub.w   d0,(a6)
            mixer.set_period(chn, datach.n_0_current_note as f64);  // move.w  (a6),6(a5)
        }
        // pit3
    }

    //------------------------------------------------
    // handle a further step of 16tel data
    //------------------------------------------------

    fn replaystep(&mut self, module: &M15Data) {  // ** work next pattern-step
        self.timpos = 0;
        let pat = match module.pattern_in_position(self.trkpos as usize) {
            Some(val) => val,
            None      => return,
        }
        
        for chn in 0..4 {
            self.mt_playvoice(pat, chn, &module, &mut mixer);
        }

        // rep5
        self.patpos += 1;                    // next step
        if self.patpos == 64 {               // pattern finished ?
            self.patpos = 0;
            self.trkpos += 1;                // next pattern in table
            if self.trkpos == self.numpat {  // song finished ?
                self.trkpos = 0;
            }
        }
    }

    //------------------------------------------------
    // proof chanel for actions
    //------------------------------------------------

    fn chanelhandler(&mut self) {
        let event = module.patterns.event(pat, self.mt_patpos, chn);
        {
            let datachn = &mut self.datachn[chn];
    
            datachn.n_0_current_note = event.note;          // get period & action-word
            datachn.n_2_sound_number = event.cmd;
            datachn.n_3_effect_number = event.cmdlo;
    
            let ins = (event.cmd >> 4) as usize;            // get nibble for soundnumber
            if ins != 0 {
                let instrument = &module.instruments[ins as usize - 1];
                datachn.n_8_soundlength = instrument.size;  // store sample-len in words
                datachn.n_18_volume = instrument.volume;    // store sample-volume
                mixer.set_volume(chn, (datach.n_18_volume as usize) << 4);  // change chanel-volume
                datachn.n_10_repeatstart = instrument.repeat as u32;        // store repeatstart
                datachn.n_14_repeatlength = instrument.replen;              // store repeatlength
                if instrument.replen != 1 {
                    datachn.n_10_repeatstart = 0;                 // repstart  = sndstart
                    datachn.n_8_soundlength = instrument.replen;  // replength = sndlength
                }
            }
        }
        // chan2
        if self.datachn[chn].n_0_current_note != 0 {
            self.datachn[chn].n_16_last_saved_note = self.datachn[chn].n_0_note;
            if ins != 0 { self.set_patch(chn, ins as usize - 1 }
            let datachn = &mut self.datachn[chn];
            mixer.set_loop_start(chn, datachn.n_10_repeatstart as usize);
            mixer.set_loop_end(chn, (datachn.n_10_repeatstart + datachn.n_14_repeatlength) as usize);
            mixer.set_period(chn, self.n_0_note as f64);
            datachn.n_20_volume_trigger = datachn.n_18_volume;
        }
        // chan4 
    }
}

impl FormatPlayer for ModPlayer {
    fn start(&mut self, data: &mut PlayerData, _mdata: &ModuleData, mixer: &mut Mixer) {
        data.speed = 6;
        data.tempo = 125;

        let pan = match self.options.option_int("pan") {
            Some(val) => val,
            None      => 70,
        };
        let panl = -128 * pan / 100;
        let panr = 127 * pan / 100;

        mixer.set_pan(0, panl);
        mixer.set_pan(1, panr);
        mixer.set_pan(2, panr);
        mixer.set_pan(3, panl);
    }

    fn play(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<ModData>().unwrap();

        self.mt_songpos = data.pos as u8;
        self.mt_pattpos = data.row as u8;
        self.timpos = data.frame as u8;

        self.replay_muzak(&module, &mut mixer);

        data.frame = self.timpos as usize;
        data.row = self.mt_pattpos as usize;
        data.pos = self.mt_songpos as usize;
        data.speed = self.mt_speed as usize;
        data.tempo = 125;
    }

    fn reset(&mut self) {
        self.mt_speed   = 6;
        self.timpos = 0;
        self.mt_songpos = 0;
        self.mt_break   = false;
        self.mt_pattpos = 0;
    }
}


#[derive(Clone,Default)]
struct DataChnx {
    n_0_current_note    : u16,
    n_2_sound_number    : u8,
    n_3_effect_number   : u8,
    //n_4_soundstart      : u32,
    n_8_soundlength     : u16,
    n_10_repeatstart    : u32,
    n_14_repeatlength   : u16,
    n_16_last_saved_note: i16,
    n_18_volume         : i16,
    n_20_volume_trigger : i16,
    //n_22_dma_bit        : u16,
}

impl DataChnx {
    pub fn new() -> Self {
        Default::default()
    }
}

static NOTETABLE: [u7, 37] = [
    856, 808, 762, 720, 678, 640, 604, 570,
    538, 508, 480, 453, 428, 404, 381, 360,
    339, 320, 302, 285, 269, 254, 240, 226,
    214, 202, 190, 180, 170, 160, 151, 143,
    135, 127, 120, 113, 000
];

