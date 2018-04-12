use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer, State};
use player::scan::SaveRestore;
use format::st::StData;
use mixer::Mixer;

/// D.O.C SoundTracker V2.0 replayer
///
/// An oxdz player based on the D.O.C SoundTracker V2.0 playroutine - Improved
/// and "omptimized" by Unknown of D.O.C, Based on the playroutine from TJC.

#[derive(SaveRestore)]
pub struct StPlayer {
    options: Options,

    mt_speed     : u8,
    mt_partnote  : u8,
    mt_partnrplay: u8,
    mt_counter   : u8,
    mt_maxpart   : u16,
    mt_status    : bool,
    mt_sample1   : [u32; 31],
    mt_audtemp   : [AudTemp; 4],
}

impl StPlayer {
    pub fn new(module: &Module, options: Options) -> Self {

        let module = module.data.as_any().downcast_ref::<StData>().unwrap();

        StPlayer {
            options,

            mt_speed     : 6,
            mt_partnote  : 0,
            mt_partnrplay: 0,
            mt_counter   : 0,
            mt_maxpart   : module.song_length as u16,
            mt_status    : false,
            mt_sample1   : [0; 31],
            mt_audtemp   : [AudTemp::new(); 4],
        }
    }

    fn mt_music(&mut self, module: &StData, mut mixer: &mut Mixer) {
        self.mt_counter += 1;
        // mt_cool
        if self.mt_counter == self.mt_speed {
            self.mt_counter = 0;
            self.mt_rout2(&module, &mut mixer);
        }

        // mt_notsix
        for chn in 0..4 {
            // mt_arpout
            let cmd = self.mt_audtemp[chn].n_2_cmd & 0xf;
            match cmd {
                0x0 => self.mt_arpegrt(chn, &mut mixer),
                0x1 => self.mt_portup(chn, &mut mixer),
                0x2 => self.mt_portdown(chn, &mut mixer),
                _   => (),
            }
        }
    }

    fn mt_portup(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_audtemp[chn];
        ch.n_22_last_note -= ch.n_3_cmdlo as i16;         // move.b  3(a6),d0 / sub.w   d0,22(a6)
        if ch.n_22_last_note < 0x71 {                     // cmp.w   #$71,22(a6) / bpl.s   mt_ok1
            ch.n_22_last_note = 0x71;                     // move.w  #$71,22(a6)
        }
        // mt_ok1
        mixer.set_period(chn, ch.n_22_last_note as f64);  // move.w  22(a6),6(a5)
    }

    fn mt_portdown(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_audtemp[chn];
        ch.n_22_last_note += ch.n_3_cmdlo as i16;         // move.b  3(a6),d0 / add.w   d0,22(a6)
        if ch.n_22_last_note >= 0x358 {                   // cmp.w   #$358,22(a6) / bmi.s   mt_ok2
            ch.n_22_last_note = 0x358;                    // move.w  #$358,22(a6)
        }
        // mt_ok2
        mixer.set_period(chn, ch.n_22_last_note as f64);  // move.w  22(a6),6(a5)
    }

    fn mt_arpegrt(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_audtemp[chn];
        let val = match self.mt_counter {
            1 => ch.n_3_cmdlo >> 4,    // mt_loop2
            2 => ch.n_3_cmdlo & 0x0f,  // mt_loop3
            3 => 0,                    // mt_loop4
            4 => ch.n_3_cmdlo >> 4,    // mt_loop2
            _ => ch.n_3_cmdlo & 0x0f,  // mt_loop3
        } as usize;

        // mt_cont
        for i in 0..36 {
            if ch.n_16_period == MT_ARPEGGIO[i] {
                if i + val < MT_ARPEGGIO.len() {  // oxdz: add sanity check
                    // mt_endpart
                    mixer.set_period(chn, MT_ARPEGGIO[i+val] as f64);  // move.w  d2,6(a5)
                    return
                }
            }
        }
    }

    fn mt_rout2(&mut self, module: &StData, mut mixer: &mut Mixer) {
        let pat = match module.pattern_in_position(self.mt_partnrplay as usize) {
            Some(val) => val,
            None      => return,
        };

        for chn in 0..4 {
            self.mt_playit(pat, chn, &module, &mut mixer);
            let ch = &mut self.mt_audtemp[chn];
            if ch.n_14_replen == 1 {
                mixer.set_sample_ptr(chn, ch.n_10_loopstart - ch.n_4_samplestart);
            }
        }

        // mt_voice0
        self.mt_partnote +=1;
        loop {
            if self.mt_partnote == 64 {
                // mt_higher
                self.mt_partnote = 0;
                self.mt_partnrplay += 1;
                if self.mt_partnrplay as u16 >= self.mt_maxpart {
                    self.mt_partnrplay = 0;    // clr.l   mt_partnrplay
                }
            }
            // mt_stop
            if self.mt_status {
                self.mt_status = false;
                self.mt_partnote = 64;
                continue;
            }
            break
        }
    }

    fn mt_playit(&mut self, pat: usize, chn: usize, module: &StData, mut mixer: &mut Mixer) {
        let event = module.patterns.event(pat, self.mt_partnote, chn);
        {
            let ch = &mut self.mt_audtemp[chn];

            ch.n_0_note = event.note;      // move.l  (a0,d1.l),(a6)
            ch.n_2_cmd = event.cmd;
            ch.n_3_cmdlo = event.cmdlo;

            let ins = ((event.cmd & 0xf0) >> 4) as usize;

            if ins != 0 {
                let instrument = &module.instruments[ins as usize - 1];
                ch.n_4_samplestart = self.mt_sample1[ins as usize - 1];     // move.l  (a1,d2),4(a6)
                ch.n_8_length = instrument.size;                            // move.w  (a3,d4),8(a6)
                ch.n_18_volume = instrument.volume as u8;                   // move.w  2(a3,d4),18(a6)
                let repeat = instrument.repeat as u32;                      // move.w  4(a3,d4),d3
                if repeat != 0 {                                            // tst.w   d3 / beq.s   mt_displace
                    ch.n_4_samplestart += repeat;                           // move.l  4(a6),d2 / add.l   d3,d2 / move.l  d2,4(a6)
                    ch.n_10_loopstart = ch.n_4_samplestart;                 // move.l  d2,10(a6)
                    ch.n_8_length = instrument.replen;                      // move.w  6(a3,d4),8(a6)
                    ch.n_14_replen = instrument.replen;                     // move.w  6(a3,d4),14(a6)
                    mixer.set_volume(chn, (ch.n_18_volume as usize) << 4);  // move.w  18(a6),8(a5)
                } else {
                    // mt_displace
                    ch.n_10_loopstart = ch.n_4_samplestart + repeat;        // move.l  4(a6),d2 / add.l   d3,d2 / move.l  d2,10(a6)
                    ch.n_14_replen = instrument.replen;                     // move.w  6(a3,d4),14(a6)
                    mixer.set_volume(chn, (ch.n_18_volume as usize) << 4);  // move.w  18(a6),8(a5)
                }
                mixer.enable_loop(chn, instrument.repeat != 0);
                mixer.set_loop_start(chn, ch.n_10_loopstart - ch.n_4_samplestart);
                mixer.set_loop_end(chn, ch.n_10_loopstart + ch.n_14_replen as u32 * 2);
            }

            // mt_nosamplechange
            if ch.n_0_note != 0 {
                ch.n_16_period = ch.n_0_note as i16;
                mixer.set_sample_ptr(chn, ch.n_4_samplestart);    // move.l  4(a6),(a5)
                mixer.set_period(chn, ch.n_0_note as f64);        // move.w  (a6),6(a5)
            }
            // mt_retrout
            if ch.n_0_note != 0 {
                ch.n_22_last_note = ch.n_0_note as i16; 
            }
        }

        // mt_nonewper
        match self.mt_audtemp[chn].n_2_cmd & 0x0f {
            0xb => self.mt_posjmp(chn),
            0xc => self.mt_setvol(chn, &mut mixer),
            0xd => self.mt_break(),
            0xe => self.mt_setfil(chn, &mut mixer),
            0xf => self.mt_setspeed(chn),
            _   => {},
        }
    }

    fn mt_posjmp(&mut self, chn: usize) {
        let ch = &mut self.mt_audtemp[chn];
        self.mt_status = !self.mt_status;
        self.mt_partnrplay = ch.n_3_cmdlo;
        self.mt_partnrplay.wrapping_sub(1);
    }

    fn mt_setvol(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_audtemp[chn];
        mixer.set_volume(chn, (ch.n_3_cmdlo as usize) << 4);  // move.b  3(a6),8(a5)
    }

    fn mt_break(&mut self) {
        self.mt_status = !self.mt_status;
    }

    fn mt_setfil(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_audtemp[chn];
        mixer.enable_filter(ch.n_3_cmdlo & 0x0f != 0);
    }

    fn mt_setspeed(&mut self, chn: usize) {
        let ch = &mut self.mt_audtemp[chn];
        if ch.n_3_cmdlo & 0x0f != 0 {
            self.mt_counter = 0;            // clr.l   mt_counter
            self.mt_speed = ch.n_3_cmdlo;   // move.b  d0,mt_cool+5
        }
        // mt_back
    }
}


#[derive(Clone,Copy,Default)]
struct AudTemp {
    n_0_note        : u16,
    n_2_cmd         : u8,
    n_3_cmdlo       : u8,
    n_4_samplestart : u32,
    n_8_length      : u16,
    n_10_loopstart  : u32,
    n_14_replen     : u16,
    n_16_period     : i16,
    n_18_volume     : u8,
    n_22_last_note  : i16,
}

impl AudTemp {
    pub fn new() -> Self {
        Default::default()
    }
}

lazy_static! {
    static ref MT_ARPEGGIO: Box<[i16; 39]> = Box::new([
        0x0358, 0x0328, 0x02fa, 0x02d0, 0x02a6, 0x0280, 0x025c,
        0x023a, 0x021a, 0x01fc, 0x01e0, 0x01c5, 0x01ac, 0x0194, 0x017d,
        0x0168, 0x0153, 0x0140, 0x012e, 0x011d, 0x010d, 0x00fe, 0x00f0,
        0x00e2, 0x00d6, 0x00ca, 0x00be, 0x00b4, 0x00aa, 0x00a0, 0x0097,
        0x008f, 0x0087, 0x007f, 0x0078, 0x0071, 0x0000, 0x0000, 0x0000
    ]);
}


impl FormatPlayer for StPlayer {
    fn start(&mut self, data: &mut PlayerData, mdata: &ModuleData, mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<StData>().unwrap();

        for i in 0..15 {
            self.mt_sample1[i] = module.samples[i].address;
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

        self.mt_music(&module, &mut mixer);

        data.frame = self.mt_counter as usize;
        data.row = self.mt_partnote as usize;
        data.pos = self.mt_partnrplay as usize;
        data.speed = self.mt_speed as usize;
        data.time += 20.0 * 125.0 / data.tempo as f32;
    }

    fn reset(&mut self) {
        self.mt_speed   = 6;
        self.mt_counter = 0;
        self.mt_partnrplay = 0;
        self.mt_status   = false;
        self.mt_partnote = 0;
    }

    unsafe fn save_state(&self) -> State {
        self.save()
    }

    unsafe fn restore_state(&mut self, state: &State) {
        self.restore(&state)
    }
}
