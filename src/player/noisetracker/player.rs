use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer};
use format::mk::ModData;
use mixer::Mixer;

/// NT1.1 Replayer
///
/// An oxdz player based on the Noisetracker V1.1 play routine by Pex "Mahoney"
/// Tufvesson and Anders “Kaktus” Berkeman (Mahoney & Kaktus - HALLONSOFT 1989).

pub struct ModPlayer {
    state  : Vec<ChannelData>,
    options: Options,

    mt_speed  : u8,
    mt_songpos: u8,
    mt_pattpos: u8,
    mt_counter: u8,
    mt_break  : bool,
}

impl ModPlayer {
    pub fn new(module: &Module, options: Options) -> Self {
        ModPlayer {
            state: vec![ChannelData::new(); 4],
            options,

            mt_speed  : 6,
            mt_songpos: 0,
            mt_pattpos: 0,
            mt_counter: 0,
            mt_break  : false,
        }
    }

    fn mt_music(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        self.mt_counter += 1;
        if self.mt_speed > self.mt_counter {
            // mt_nonew
            for chn in 0..4 {
                self.mt_checkcom(chn, &mut mixer);
            }
            // mt_endr
            if self.mt_break {
                self.mt_nex(&module);
            }
            return;
        }

        self.mt_counter = 0;
        self.mt_getnew(&module, &mut mixer);
    }

    fn mt_arpeggio(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        let val = match self.mt_counter % 3 {
            2 => {  // mt_arp1
                     state.n_3_cmdlo & 15
                 },
            0 => {  // mt_arp2
                     0
                 },
            _ => {
                     state.n_3_cmdlo >> 4
                 },
        } as usize;

        // mt_arp3
        for i in 0..36 {
            if state.n_10_period >= MT_PERIODS[i + val] {
                // mt_arp4
                mixer.set_period(chn, state.n_10_period as f64);  // move.w  d2,$6(a5)
                return
            }
        }
    }

    fn mt_getnew(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        let pat = match module.pattern_in_position(self.mt_songpos as usize) {
            Some(val) => val,
            None      => return,
        };

        for chn in 0..4 {
            self.mt_playvoice(pat, chn, &module, &mut mixer);
        }

        // mt_setdma
        self.mt_pattpos +=1;
        if self.mt_pattpos == 64 {
            self.mt_nex(&module);
        }
    }

    fn mt_playvoice(&mut self, pat: usize, chn: usize, module: &ModData, mut mixer: &mut Mixer) {
        let event = module.patterns.event(pat, self.mt_pattpos, chn);
        {
            let state = &mut self.state[chn];
    
            state.n_0_note = event.note;      // move.l  (a0,d1.l),(a6)
            state.n_2_cmd = event.cmd;
            state.n_3_cmdlo = event.cmdlo;
    
            let ins = (((event.note & 0xf000) >> 8) | ((event.cmd as u16 & 0xf0) >> 4)) as usize;
    
            if ins != 0 {
                let instrument = &module.instruments[ins as usize - 1];
                state.n_8_length = instrument.size;                            // move.w  (a3,d4.l),$8(a6)
                state.n_12_volume = instrument.volume as u8;                   // move.w  $2(a3,d4.l),$12(a6)
                if instrument.repeat != 0 {
                    state.n_a_loopstart = instrument.repeat as u32;
                    state.n_8_length = instrument.repeat + instrument.replen;
                    state.n_e_replen = instrument.replen;                      // move.w  $6(a3,d4.l),$e(a6)
    
                    state.n_8_length = instrument.repeat + instrument.replen;
                    mixer.set_volume(chn, (state.n_12_volume as usize) << 4);  // move.w  $12(a6),$8(a5)
                    mixer.enable_loop(chn, true);
                } else {
                    // mt_noloop
                    state.n_8_length = instrument.size;
                    state.n_e_replen = instrument.replen;
                    mixer.enable_loop(chn, false);
                    mixer.set_volume(chn, (state.n_12_volume as usize) << 4);  // move.w  $12(a6),$8(a5)
                }
                mixer.set_patch(chn, ins as usize - 1, ins as usize - 1);
                mixer.set_loop_start(chn, state.n_a_loopstart * 2);
                mixer.set_loop_end(chn, (state.n_a_loopstart + state.n_e_replen as u32) * 2);
            }
        }

        // mt_setregs
        if self.state[chn].n_0_note & 0xfff != 0 {
            if self.state[chn].n_2_cmd & 0xf == 0x3 {
                self.mt_setmyport(chn);
                self.mt_checkcom2(chn, &mut mixer)
            } else {
                self.mt_setperiod(chn, &mut mixer);
            }
        } else {
            self.mt_checkcom2(chn, &mut mixer);  // If no note
        }
    }

    fn mt_setperiod(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let state = &mut self.state[chn];
            state.n_10_period = (state.n_0_note & 0xfff) as i16;
            state.n_1b_vibpos = 0;                     // clr.b   $1b(a6)
            mixer.set_voicepos(chn, 0.0);
            mixer.set_period(chn, state.n_10_period as f64);
        }

        self.mt_checkcom2(chn, &mut mixer);
    }

    fn mt_nex(&mut self, module: &ModData) {
        self.mt_pattpos = 0;
        self.mt_break = false;
        self.mt_songpos = self.mt_songpos.wrapping_add(1);
        self.mt_songpos &= 0x7f;
        if self.mt_songpos as usize >= module.len() {  // cmp.b   mt_data+$3b6,d1
            // self.mt_songpos = 0 in Noisetracker 1.0
            self.mt_songpos = module.restart;          // move.b  mt_data+$3b7,mt_songpos
        }
    }

    fn mt_setmyport(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        state.n_18_wantperiod = (state.n_0_note & 0xfff) as i16;
        state.n_16_portdir = false;     // clr.b   $16(a6)
        if state.n_10_period == state.n_18_wantperiod {
            // mt_clrport
            state.n_18_wantperiod = 0;  // clr.w   $18(a6)
        } else if state.n_10_period < state.n_18_wantperiod {
            state.n_16_portdir = true;  // move.b  #$1,$16(a6)
        }
    }

    fn mt_myport(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if state.n_3_cmdlo != 0 {
             state.n_17_toneportspd = state.n_3_cmdlo;
             state.n_3_cmdlo = 0;
        }
        // mt_myslide
        if state.n_18_wantperiod != 0 {
            if state.n_16_portdir {
                state.n_10_period += state.n_17_toneportspd as i16;
                if state.n_10_period > state.n_18_wantperiod {
                    state.n_10_period = state.n_18_wantperiod;
                    state.n_18_wantperiod = 0;
                }
            } else {
                // mt_mysub
                state.n_10_period -= state.n_17_toneportspd as i16;
                if state.n_10_period < state.n_18_wantperiod {
                    state.n_10_period = state.n_18_wantperiod;
                    state.n_18_wantperiod = 0;
                }
            }
        }
        mixer.set_period(chn, state.n_10_period as f64);  // move.w  $10(a6),$6(a5)
    }

    fn mt_vib(&mut self, chn: usize, mixer: &mut Mixer) {
        {
            let state = &mut self.state[chn];
            if state.n_3_cmdlo != 0 {
                state.n_1a_vibrato = state.n_3_cmdlo;

                let pos = (state.n_1b_vibpos >> 2) & 0x1f;
                let val = MT_SIN[pos as usize];
                let amt = ((val as usize * (state.n_1a_vibrato & 0xf) as usize) >> 6) as i16;

                let mut period = state.n_10_period;
                if state.n_1b_vibpos & 0x80 == 0 {
                    period += amt
                } else {
                    // mt_vibmin
                    period -= amt
                }

                mixer.set_period(chn, period as f64);
                state.n_1b_vibpos = state.n_1b_vibpos.wrapping_add((state.n_1a_vibrato >> 2) & 0x3c);
            }
        }
    }

    fn mt_checkcom(&mut self, chn: usize, mut mixer: &mut Mixer) {
        let cmd = self.state[chn].n_2_cmd & 0xf;
        match cmd {
            0x0 => self.mt_arpeggio(chn, &mut mixer),
            0x1 => self.mt_portup(chn, &mut mixer),
            0x2 => self.mt_portdown(chn, &mut mixer),
            0x3 => self.mt_myport(chn, &mut mixer),
            0x4 => self.mt_vib(chn, &mut mixer),
            _   => {
                       mixer.set_period(chn, self.state[chn].n_10_period as f64);  // move.w  $10(a6),$6(a5)
                       match cmd {
                           0xa => self.mt_volslide(chn, &mut mixer),
                           _   => {},
                       }
                   }
        }
    }

    fn mt_volslide(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if state.n_3_cmdlo >> 4 == 0 {
            // mt_voldown
            let cmdlo = state.n_3_cmdlo & 0x0f;
            if state.n_12_volume > cmdlo {
                state.n_12_volume -= cmdlo;
            } else {
                state.n_12_volume = 0;
            }
        } else {
            state.n_12_volume += state.n_3_cmdlo >> 4;
            if state.n_12_volume > 0x40 {
                state.n_12_volume = 0x40;
            }
        }
        // mt_vol2
        mixer.set_volume(chn, (state.n_12_volume as usize) << 4);  // move.w  $12(a6),$8(a5)
    }

    fn mt_portup(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        state.n_10_period -= state.n_3_cmdlo as i16;
        if (state.n_10_period & 0xfff) < 0x71 {
            state.n_10_period &= 0xf000;
            state.n_10_period |= 0x71;
        }
        // mt_por2
        mixer.set_period(chn, (state.n_10_period & 0xfff) as f64);  // move.w $10(a6),d0; and.w #$fff,d0; move.w d0,$6(a5)
    }

    fn mt_portdown(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        state.n_10_period += state.n_3_cmdlo as i16;
        if (state.n_10_period & 0xfff) >= 0x358 {
            state.n_10_period &= 0xf000;
            state.n_10_period |= 0x358;
        }
        mixer.set_period(chn, (state.n_10_period & 0xfff) as f64);  // move.w $10(a6),d0; and.w #$fff,d0; move.w d0,$6(a5)
    }

    fn mt_checkcom2(&mut self, chn: usize, mut mixer: &mut Mixer) {
        match self.state[chn].n_2_cmd & 0xf {
            0xe => self.mt_setfilt(),
            0xd => self.mt_pattbreak(),
            0xb => self.mt_posjmp(chn),
            0xc => self.mt_setvol(chn, &mut mixer),
            0xf => self.mt_setspeed(chn),
            _   => {},
        }
    }

    fn mt_setfilt(&self) {
    }

    fn mt_pattbreak(&mut self) {
        self.mt_break = !self.mt_break;
    }

    fn mt_posjmp(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        self.mt_songpos = state.n_3_cmdlo.wrapping_sub(1);
        self.mt_break = !self.mt_break;
    }

    fn mt_setvol(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if state.n_3_cmdlo > 0x40 {  // cmp.b   #$40,$3(a6)
            state.n_3_cmdlo = 40     // move.b  #$40,$3(a6)
        }
        // mt_vol4
        mixer.set_volume(chn, (state.n_3_cmdlo as usize) << 4);  // move.b  $3(a6),$8(a5)
    }

    fn mt_setspeed(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        if state.n_3_cmdlo > 0x1f {  // cmp.b   #$1f,$3(a6)
            state.n_3_cmdlo = 0x1f;  // move.b  #$1f,$3(a6)
        }
        // mt_sets
        if state.n_3_cmdlo != 0 {
            self.mt_speed = state.n_3_cmdlo;  // move.b  d0,mt_speed
            self.mt_counter = 0;            // clr.b   mt_counter
        }
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
        self.mt_counter = data.frame as u8;

        self.mt_music(&module, &mut mixer);

        data.frame = self.mt_counter as usize;
        data.row = self.mt_pattpos as usize;
        data.pos = self.mt_songpos as usize;
        data.speed = self.mt_speed as usize;
        data.tempo = 125;
    }

    fn reset(&mut self) {
        self.mt_speed   = 6;
        self.mt_counter = 0;
        self.mt_songpos = 0;
        self.mt_break   = false;
        self.mt_pattpos = 0;
    }
}


#[derive(Clone,Default)]
struct ChannelData {
    n_0_note        : u16,
    n_2_cmd         : u8,
    n_3_cmdlo       : u8,
    //n_4_samplestart: u32,
    n_8_length      : u16,
    n_a_loopstart   : u32,
    n_e_replen      : u16,
    n_10_period     : i16,
    n_12_volume     : u8,
    n_16_portdir    : bool,
    n_17_toneportspd: u8,
    n_18_wantperiod : i16,
    n_1a_vibrato    : u8,
    n_1b_vibpos     : u8,
}

impl ChannelData {
    pub fn new() -> Self {
        Default::default()
    }
}


static MT_SIN: [u8; 32] = [
    0x00, 0x18, 0x31, 0x4a, 0x61, 0x78, 0x8d, 0xa1, 0xb4, 0xc5, 0xd4, 0xe0, 0xeb, 0xf4, 0xfa, 0xfd,
    0xff, 0xfd, 0xfa, 0xf4, 0xeb, 0xe0, 0xd4, 0xc5, 0xb4, 0xa1, 0x8d, 0x78, 0x61, 0x4a, 0x31, 0x18
];

static MT_PERIODS: [i16; 38] = [
    0x0358, 0x0328, 0x02fa, 0x02d0, 0x02a6, 0x0280, 0x025c, 0x023a, 0x021a, 0x01fc, 0x01e0,
    0x01c5, 0x01ac, 0x0194, 0x017d, 0x0168, 0x0153, 0x0140, 0x012e, 0x011d, 0x010d, 0x00fe,
    0x00f0, 0x00e2, 0x00d6, 0x00ca, 0x00be, 0x00b4, 0x00aa, 0x00a0, 0x0097, 0x008f, 0x0087,
    0x007f, 0x0078, 0x0071, 0x0000, 0x0000
];
