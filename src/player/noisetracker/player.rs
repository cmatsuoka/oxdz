use module::{Module, ModuleData};
use player::{PlayerData, FormatPlayer};
use format::mk::{ModData, PeriodTable};
use mixer::Mixer;

/// PT2.1A Replayer
///
/// An oxdz player based on the Noisetracker V1.1 play routine by Pex "Mahoney"
/// Tufvesson and Anders “Kaktus” Berkeman (Mahoney & Kaktus - HALLONSOFT 1989).
///
/// Notes:
/// * Mixer volumes are *16, so adjust when setting.
/// * Pattern periods are decoded beforehand and stored as a note value.
/// * Pattern instruments are decoded beforehand and stored in channel state.

pub struct ModPlayer {
    state : Vec<ChannelData>,

    mt_speed  : u8,
    mt_songpos: u8,
    mt_pattpos: u8,
    mt_counter: u8,
    mt_break  : bool,
}

impl ModPlayer {
    pub fn new(module: &Module) -> Self {
        ModPlayer {
            state: vec![ChannelData::new(); module.data.channels()],

            mt_speed          : 6,
            mt_counter        : 0,
            mt_songpos       : 0,
            mt_break     : 0,
            mt_pos_jump_flag  : false,
            mt_pbreak_flag    : false,
            mt_low_mask       : 0,
            mt_patt_del_time  : 0,
            mt_patt_del_time_2: 0,
            mt_pattpos    : 0,
            cia_tempo         : 125,
        }
    }

    fn mt_music(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        self.mt_counter += 1;
        if self.mt_speed > self.mt_counter {
            // mt_nonew
            for chn in 0..module.channels() {
                mt_checkcom(chn, &mut mixer);
            }
            return;
        }

        self.mt_counter = 0;
        self.mt_getnew(&module, &mut mixer);

/*
        if self.mt_patt_del_time_2 == 0 {
            self.mt_getnew(&module, &mut mixer);
        } else {
            self.mt_no_new_all_channels(&module, &mut mixer);
        }

        // mt_dskip
        self.mt_pattpos +=1;
        if self.mt_patt_del_time != 0 {
            self.mt_patt_del_time_2 = self.mt_patt_del_time;
            self.mt_patt_del_time = 0;
        }

        // mt_dskc
        if self.mt_patt_del_time_2 != 0 {
            self.mt_patt_del_time_2 -= 1;
            if self.mt_patt_del_time_2 != 0 {
                self.mt_pattpos -= 1;
            }
        }

        // mt_dska
        if self.mt_pbreak_flag {
            self.mt_pbreak_flag = false;
            self.mt_pattpos = self.mt_break;
            self.mt_break = 0;
        }

        // mt_nnpysk
        if self.mt_pattpos >= 64 {
            self.mt_nex(&module);
        }
        //self.mt_no_new_pos_yet(&module);
*/
    }

    fn mt_arpeggio(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        let val = match self.mt_counter % 3 {
            2 => {  // mt_arp1
                     state.n_cmdlo & 15
                 },
            0 => {  // mt_arp2
                     0
                 },
            _ => {
                     state.n_cmdlo >> 4
                 },
        } as u8;
        // mt_arp3
        // mt_arp4
        let note = PeriodTable::period_to_note(state.n_period, state.n_finetune);
        let period = PeriodTable::note_to_period(note + val, state.n_finetune);
        mixer.set_period(chn, period as f64);  // move.w  d2,$6(a5)
    }

    fn mt_getnew(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        let pat = match module.pattern_in_position(self.mt_songpos as usize) {
            Some(val) => val,
            None      => return,
        };

        for chn in 0..module.channels() {
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
        let (note, ins, cmd, cmdlo) = (event.note, event.ins, event.cmd, event.cmdlo);

        if { let e = &self.state[chn]; e.n_note | e.n_ins | e.n_cmd | e.n_cmdlo == 0 } {  // TST.L   (A6)
            self.per_nop(chn, &mut mixer);
        }

        {
            let state = &mut self.state[chn];

            // mt_plvskip
            state.n_note = note;
            state.n_ins = ins;
            state.n_cmd = cmd;
            state.n_cmdlo = cmdlo;

            if ins != 0 {
                let instrument = &module.instruments[ins as usize - 1];
                state.n_length = instrument.size;
                //state.n_reallength = instrument.size;
                state.n_finetune = instrument.finetune as i8;
                state.n_volume = instrument.volume as u8;
                mixer.set_patch(chn, ins as usize - 1, ins as usize - 1);
                mixer.set_volume(chn, (instrument.volume as usize) << 4);
                if instrument.replen > 1 {
                    state.n_loopstart = instrument.repeat as u32;
                    state.n_wavestart = instrument.repeat as u32;
                    state.n_length = instrument.repeat + instrument.replen;
                    state.n_replen = instrument.replen;
                    mixer.set_loop_start(chn, state.n_loopstart * 2);
                    mixer.set_loop_end(chn, (state.n_loopstart + state.n_replen as u32) * 2);
                    mixer.enable_loop(chn, true);
                } else {
                    // mt_NoLoop
                    state.n_length = instrument.repeat + instrument.replen;
                    state.n_replen = instrument.replen;
                    mixer.enable_loop(chn, false);
                }
            }
        }

        self.mt_setregs(chn, &mut mixer);
    }

    fn mt_setregs(&mut self, chn: usize, mut mixer: &mut Mixer) {
        if self.state[chn].n_note != 0 {
            if self.state[chn].n_cmd & 0x0f == 0x03 {
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
            let period = PeriodTable::note_to_period(state.n_note, state.n_finetune);
            state.n_period = period;
            state.n_vibratopos = 0;
            mixer.set_voicepos(chn, 0.0);
            mixer.set_period(chn, state.n_period as f64);
        }

        self.mt_checkcom2(chn, &mut mixer);
    }

    fn mt_nex(&mut self, module: &ModData) {
        self.mt_pattpos = 0;
        self.mt_break = 0;
        self.mt_songpos = self.mt_songpos.wrapping_add(1);
        self.mt_songpos &= 0x7f;
        if self.mt_songpos as usize >= module.len() {  // cmp.b   mt_data+$3b6,d1
            self.mt_songpos = module.rst;              // move.b  mt_data+$3b7,mt_songpos
        }
        // movem.l (a7)+,d0-d4/a0-a3/a5-a6
    }

    fn mt_setmyport(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        state.n_wantedperiod = PeriodTable::note_to_period(state.n_note, state.n_finetune);
        state.n_toneportdirec = false;     // clr.b   $16(a6)
        if state.n_period == state.n_wantedperiod {
            // mt_clrport
            state.n_wantedperiod = 0;      // clr.w   $18(a6)
        } else if state.n_period < state.n_wantedperiod {
            state.n_toneportdirec = true;  // move.b  #$1,$16(a6)
        }
    }

    fn mt_vib(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let state = &mut self.state[chn];
            if state.n_cmdlo != 0 {
                if state.n_cmdlo & 0x0f != 0 {
                    state.n_vibratocmd = (state.n_vibratocmd & 0xf0) | (state.n_cmdlo & 0x0f)
                }
                // mt_vibskip
                if state.n_cmdlo & 0xf0 != 0 {
                    state.n_vibratocmd = (state.n_vibratocmd & 0x0f) | (state.n_cmdlo & 0xf0)
                }
                // mt_vibskip2
            }
        }
        self.mt_vib_2(chn, &mut mixer);
    }

    fn mt_vib_2(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        let mut pos = (state.n_vibratopos >> 2) & 0x1f;
        let val = match state.n_wavecontrol & 0x03 {
            0 => {  // mt_vib_sine
                     MT_SIN[pos as usize]
                 },
            1 => {  // mt_vib_rampdown
                     pos <<= 3;
                     if pos & 0x80 != 0 { 255 - pos } else { pos }
                 },
            _ => {
                     255
                 }
        };
        // mt_vib_set
        let mut period = state.n_period;
        let amt = (val as usize * (state.n_vibratocmd & 15) as usize) >> 7;
        if state.n_vibratopos & 0x80 == 0 {
            period += amt as u16
        } else {
            period -= amt as u16
        };

        // mt_Vibrato3
        mixer.set_period(chn, period as f64);
        state.n_vibratopos = state.n_vibratopos.wrapping_add((state.n_vibratocmd >> 2) & 0x3c);
    }

    fn mt_checkcom(&mut self, chn: usize, mut mixer: &mut Mixer) {
        let cmd = self.state[chn].n_cmd;
        match cmd {
            0x0 => self.mt_arpeggio(chn, &mut mixer),
            0x1 => self.mt_portup(chn, &mut mixer),
            0x2 => self.mt_portdown(chn, &mut mixer),
            0x3 => self.mt_myport(chn, &mut mixer),
            0x4 => self.mt_vib(chn, &mut mixer),
            _   => {
                       mixer.set_period(chn, self.state[chn].n_period as f64);  // move.w  $10(a6),$6(a5)
                       match cmd {
                           0xa => self.mt_volslide(chn, &mut mixer),
                           _   => {},
                       }
                   }
        }
    }

    fn mt_volslide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        if self.state[chn].n_cmdlo >> 4 == 0 {
            // mt_voldown
            let cmdlo = state.n_cmdlo & 0x0f;
            if state.n_volume > cmdlo {
                state.n_volume -= cmdlo;
            } else {
                state.n_volume = 0;
            }
        } else {
            state.n_volume += state.n_cmdlo >> 4;
            if state.n_volume > 0x40 {
                state.n_volume = 0x40;
            }
        }
        // mt_vol2
        mixer.set_volume(chn, (state.n_volume as usize) << 4);  // move.w  $12(a6),$8(a5)
    }

    fn mt_portup(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        state.n_period -= (state.n_cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if state.n_period < 113 {
            state.n_period = 113;
        }
        mixer.set_period(chn, state.n_period as f64);  // MOVE.W  n_period(A6),6(A5)
    }

    fn mt_portdown(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        state.n_period += (state.n_cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if state.n_period > 856 {
            state.n_period = 856;
        }
        mixer.set_period(chn, state.n_period as f64);  // MOVE.W  D0,6(A5)
    }

    fn mt_myport(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let state = &mut self.state[chn];
            if state.n_cmdlo != 0 {
                state.n_toneportspeed = state.n_cmdlo;
                state.n_cmdlo = 0;
            }
        }
        self.mt_tone_port_no_change(chn, &mut mixer);
    }

    fn mt_tone_port_no_change(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if state.n_wantedperiod == 0 {
            return;
        }
        if state.n_toneportdirec {
            // mt_TonePortaDown
            state.n_period += state.n_toneportspeed as u16;
            if state.n_period > state.n_wantedperiod {
                state.n_period = state.n_wantedperiod;
                state.n_wantedperiod = 0;
            }
        } else {
            // mt_TonePortaUp
            if state.n_period > state.n_toneportspeed as u16 {
                state.n_period -= state.n_toneportspeed as u16;
            } else {
                state.n_period = 0;
            }
            if state.n_period < state.n_wantedperiod {
                state.n_period = state.n_wantedperiod;
                state.n_wantedperiod = 0;
            }
        }
        // mt_TonePortaSetPer
        if state.n_glissfunk & 0x0f != 0 {
        }
        // mt_GlissSkip
        mixer.set_period(chn, state.n_period as f64);
    }

    fn mt_checkcom2(&mut self, chn: usize, mut mixer: &mut Mixer) {
        // mt_UpdateFunk()

        match self.state[chn].n_cmd {
            0xe => self.mt_setfilt(),
            0xd => self.mt_pattbreak(chn),
            0xb => self.mt_posjmp(chn),
            0xc => self.mt_setvol(chn, &mut mixer),
            0xf => self.mt_setspeed(chn),
            _   => {},
        }
    }

    fn mt_setfilt(&self) {
    }

    fn mt_pattbreak(&mut self, chn: usize) {
        self.mt_break = !self.mt_break;
    }

    fn mt_posjmp(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        self.mt_songpos = state.n_cmdlo.wrapping_sub(1);
        self.mt_break = !self.mt_break;
    }

    fn mt_setvol(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if state.n_cmdlo > 0x40 {  // cmp.b   #$40,$3(a6)
            state.n_cmdlo = 40     // move.b  #$40,$3(a6)
        }
        // mt_vol4
        mixer.set_volume(chn, (state.n_cmdlo as usize) << 4);  // move.b  $3(a6),$8(a5)
    }

    fn mt_setspeed(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        if state.n_cmdlo > 0x1f {  // cmp.b   #$1f,$3(a6)
            state.n_cmdlo = 0x1f;  // move.b  #$1f,$3(a6)
        }
        // mt_sets
        if state.n_cmdlo != 0 {
            self.mt_speed = state.n_cmdlo;  // move.b  d0,mt_speed
            self.mt_counter = 0;            // clr.b   mt_counter
        }
    }
}

impl FormatPlayer for ModPlayer {
    fn start(&mut self, data: &mut PlayerData, _mdata: &ModuleData, _mixer: &mut Mixer) {
        data.speed = 6;
        data.tempo = 125;
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
        data.tempo = self.cia_tempo as usize;
    }

    fn reset(&mut self) {
        self.mt_speed           = 6;
        self.mt_counter         = 0;
        self.mt_songpos        = 0;
        self.mt_break      = false;
        self.mt_pos_jump_flag   = false;
        self.mt_pbreak_flag     = false;
        self.mt_low_mask        = 0;
        self.mt_patt_del_time   = 0;
        self.mt_patt_del_time_2 = 0;
        self.mt_pattpos     = 0;
    }
}


#[derive(Clone,Default)]
struct ChannelData {
    n_note         : u8,
    n_ins          : u8,   // not in PT2.1A
    n_cmd          : u8,
    n_cmdlo        : u8,
    n_length       : u16,
    n_loopstart    : u32,
    n_replen       : u16,
    n_period       : u16,
    n_finetune     : i8,
    n_volume       : u8,
    n_toneportdirec: bool,
    n_toneportspeed: u8,
    n_wantedperiod : u16,
    n_vibratocmd   : u8,
    n_vibratopos   : u8,
    n_tremolocmd   : u8,
    n_tremolopos   : u8,
    n_wavecontrol  : u8,
    n_glissfunk    : u8,
    n_sampleoffset : u8,
    n_pattpos      : u8,
    n_loopcount    : u8,
    n_funkoffset   : u8,
    n_wavestart    : u32,
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

