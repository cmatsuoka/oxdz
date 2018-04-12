use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer, State};
use player::scan::SaveRestore;
use format::st::StData;
use mixer::Mixer;

/// Ultimate Soundtracker V27 replayer
///
/// An oxdz player based on the Ultimate Soundtracker V27, written 1987/1988 by
/// Karsten Obarski. "All bugs removed".
///
/// > "Just look at it -- so small, innocent and cute. :)"
/// > -- Olav "8bitbubsy" Sørensen

#[derive(SaveRestore)]
pub struct USTPlayer {
    options   : Options,

    datachn   : [DataChnx; 4],
    pointers  : [u32; 15],
    //lev6save  : u32,
    trkpos    : u16,  // u32,
    patpos    : u8,   // u32,
    numpat    : u16,
    //enbits    : u16,
    timpos    : u16,
}

impl USTPlayer {
    pub fn new(module: &Module, options: Options) -> Self {

        let module = module.data.as_any().downcast_ref::<StData>().unwrap();

        USTPlayer {
            options,

            datachn : [DataChnx::new(); 4],
            pointers: [0; 15],
            trkpos  : 0,
            patpos  : 0,
            numpat  : module.song_length as u16,
            timpos  : 0,
        }
    }

    //------------------------------------------------
    // replay-routine
    //------------------------------------------------

    fn replay_muzak(&mut self, module: &StData, mut mixer: &mut Mixer) {
        self.timpos += 1;
        if self.timpos == 6 {
            self.replaystep(&module, &mut mixer)
        } else {
            //------------------------------------------------
            // time left to handle effects between steps
            //------------------------------------------------

            // chaneleffects
            for chn in 0..4 {
                if self.datachn[chn].n_3_effect_number != 0 {
                    self.ceff5(chn, &mut mixer)
                }
            }
        }
    }

    fn ceff5(&mut self, chn: usize, mut mixer: &mut Mixer) {
        match self.datachn[chn].n_2_sound_number & 0x0f {
            1 => self.arpreggiato(chn, &mut mixer),
            2 => self.pitchbend(chn, &mut mixer),
            _ => (),
        }
    }

    //------------------------------------------------
    // effect 1 arpreggiato
    //------------------------------------------------

    fn arpreggiato(&mut self, chn: usize, mixer: &mut Mixer) {  // ** spread by time
        let datachn = &mut self.datachn[chn];
        let val = match self.timpos {  // ** get higher note-values or play original
            1 => datachn.n_3_effect_number >> 4,    // arp1
            2 => datachn.n_3_effect_number & 0x0f,  // arp2
            3 => 0,                                 // arp3
            4 => datachn.n_3_effect_number >> 4,    // arp1
            5 => datachn.n_3_effect_number & 0x0f,  // arp2
            _ => 0,
        } as usize;

        // arp4
        for i in 0..36 {
            if datachn.n_16_last_saved_note == NOTETABLE[i] {
                if i + val < NOTETABLE.len() {  // oxdz: add sanity check
                    // mt_endpart
                    mixer.set_period(chn, NOTETABLE[i+val] as f64);  // move.w  d2,6(a5)
                    return
                }
            }
        }
    }

    //------------------------------------------------
    // effect 2 pitchbend
    //------------------------------------------------

    fn pitchbend(&mut self, chn: usize, mixer: &mut Mixer) {
        let datachn = &mut self.datachn[chn];
        let val = (datachn.n_3_effect_number >> 4) as i16;
        if val != 0 {
            datachn.n_0_note += val;                         // add.w   d0,(a6)
            mixer.set_period(chn, datachn.n_0_note as f64);  // move.w  (a6),6(a5)
            return
        }
        // pit2
        let val = (datachn.n_3_effect_number & 0x0f) as i16;
        if val != 0 {
            datachn.n_0_note -= val;                         // sub.w   d0,(a6)
            mixer.set_period(chn, datachn.n_0_note as f64);  // move.w  (a6),6(a5)
        }
        // pit3
    }

    //------------------------------------------------
    // handle a further step of 16tel data
    //------------------------------------------------

    fn replaystep(&mut self, module: &StData, mut mixer: &mut Mixer) {  // ** work next pattern-step
        self.timpos = 0;
        let pat = match module.pattern_in_position(self.trkpos as usize) {
            Some(val) => val,
            None      => return,
        };

        for chn in 0..4 {
            self.chanelhandler(pat, chn, &module, &mut mixer);
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

    fn chanelhandler(&mut self, pat: usize, chn: usize, module: &StData, mixer: &mut Mixer) {
        let event = module.patterns.event(pat, self.patpos, chn);
        {
            let datachn = &mut self.datachn[chn];

            datachn.n_0_note = event.note as i16;           // get period & action-word
            datachn.n_2_sound_number = event.cmd;
            datachn.n_3_effect_number = event.cmdlo;

            let ins = (event.cmd >> 4) as usize;            // get nibble for soundnumber
            if ins != 0 {
                let instrument = &module.instruments[ins as usize - 1];
                datachn.n_4_soundstart = self.pointers[ins as usize - 1];    // store sample-address
                datachn.n_8_soundlength = instrument.size;                   // store sample-len in words
                datachn.n_18_volume = instrument.volume as i16;              // store sample-volume
                mixer.set_volume(chn, (datachn.n_18_volume as usize) << 4);  // change chanel-volume
                datachn.n_10_repeatstart = datachn.n_4_soundstart + instrument.repeat as u32;  // store repeatstart
                datachn.n_14_repeatlength = instrument.replen;               // store repeatlength
                if instrument.replen != 1 {
                    datachn.n_10_repeatstart = datachn.n_4_soundstart;       // repstart  = sndstart
                    datachn.n_8_soundlength = instrument.replen;             // replength = sndlength
                }
                mixer.enable_loop(chn, instrument.replen != 1);
            }
        }
        // chan2
        if self.datachn[chn].n_0_note != 0 {
            let datachn = &mut self.datachn[chn];
            datachn.n_16_last_saved_note = datachn.n_0_note;                 // save note for effect
            mixer.set_sample_ptr(chn, datachn.n_4_soundstart);
            mixer.set_loop_start(chn, datachn.n_10_repeatstart - datachn.n_4_soundstart);
            mixer.set_loop_end(chn, datachn.n_10_repeatstart - datachn.n_4_soundstart + datachn.n_14_repeatlength as u32 * 2);
            mixer.set_period(chn, datachn.n_0_note as f64);
            datachn.n_20_volume_trigger = datachn.n_18_volume;
        }
        // chan4 
    }
}


//------------------------------------------------
// used varibles
//------------------------------------------------
//       datachx - structure     (22 bytes)
//
//       00.w    current note
//       02.b    sound-number
//       03.b    effect-number
//       04.l    soundstart
//       08.w    soundlenght in words
//       10.l    repeatstart
//       14.w    repeatlength
//       16.w    last saved note
//       18.w    volume
//       20.w    volume trigger (note on dynamic)
//       22.w    dma-bit
//------------------------------------------------

#[derive(Clone,Copy,Default)]
struct DataChnx {
    n_0_note            : i16,
    n_2_sound_number    : u8,
    n_3_effect_number   : u8,
    n_4_soundstart      : u32,
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

lazy_static! {
    static ref NOTETABLE: Box<[i16; 37]> = Box::new([
        856, 808, 762, 720, 678, 640, 604, 570,
        538, 508, 480, 453, 428, 404, 381, 360,
        339, 320, 302, 285, 269, 254, 240, 226,
        214, 202, 190, 180, 170, 160, 151, 143,
        135, 127, 120, 113, 000
    ]);
}

impl FormatPlayer for USTPlayer {
    fn start(&mut self, data: &mut PlayerData, mdata: &ModuleData, mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<StData>().unwrap();

        for i in 0..15 {
            self.pointers[i] = module.samples[i].address;
        }

        data.speed = 6;
        data.tempo = module.tempo as f32;
        data.time  = 0.0;

        data.initial_speed = data.speed;
        data.initial_tempo = data.tempo;

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

        mixer.enable_paula(true);
    }

    fn play(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<StData>().unwrap();

        self.replay_muzak(&module, &mut mixer);

        data.frame = self.timpos as usize;
        data.row = self.patpos as usize;
        data.pos = self.trkpos as usize;
        data.time += 20.0 * 125.0 / data.tempo as f32;
    }

    fn reset(&mut self) {
        self.timpos = 0;
        self.trkpos = 0;
        self.patpos = 0;
    }

    unsafe fn save_state(&self) -> State {
        self.save()
    }

    unsafe fn restore_state(&mut self, state: &State) {
        self.restore(&state)
    }
}
