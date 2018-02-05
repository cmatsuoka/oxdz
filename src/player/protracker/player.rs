use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer};
use format::mk::ModData;
use mixer::Mixer;

/// PT2.1A Replayer
///
/// An oxdz player based on the Protracker V2.1A play routine written by Peter
/// "CRAYON" Hanning / Mushroom Studios in 1992. Original names are used whenever
/// possible (converted to snake case according to Rust convention, i.e.
/// mt_PosJumpFlag becomes mt_pos_jump_flag).
///
/// Notes:
/// * Mixer volumes are *16, so adjust when setting.
/// * CIA tempo support added to the original PT2.1A set speed command.

pub struct ModPlayer {
    state  : Vec<ChannelData>,
    options: Options, 

    mt_speed          : u8,
    mt_counter        : u8,
    mt_song_pos       : u8,
    mt_pbreak_pos     : u8,
    mt_pos_jump_flag  : bool,
    mt_pbreak_flag    : bool,
    mt_low_mask       : u8,
    mt_patt_del_time  : u8,
    mt_patt_del_time_2: u8,
    mt_pattern_pos    : u8,
    cia_tempo         : u8,
}

impl ModPlayer {
    pub fn new(module: &Module, options: Options) -> Self {
        ModPlayer {
            options,
            state: vec![ChannelData::new(); 4],

            mt_speed          : 6,
            mt_counter        : 0,
            mt_song_pos       : 0,
            mt_pbreak_pos     : 0,
            mt_pos_jump_flag  : false,
            mt_pbreak_flag    : false,
            mt_low_mask       : 0,
            mt_patt_del_time  : 0,
            mt_patt_del_time_2: 0,
            mt_pattern_pos    : 0,
            cia_tempo         : 125,
        }
    }

    fn mt_music(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        self.mt_counter += 1;
        if self.mt_speed > self.mt_counter {
            // mt_NoNewNote
            self.mt_no_new_all_channels(&module, &mut mixer);
            self.mt_no_new_pos_yet(&module);
            return
        }

        self.mt_counter = 0;
        if self.mt_patt_del_time_2 == 0 {
            self.mt_get_new_note(&module, &mut mixer);
        } else {
            self.mt_no_new_all_channels(&module, &mut mixer);
        }

        // mt_dskip
        self.mt_pattern_pos +=1;
        if self.mt_patt_del_time != 0 {
            self.mt_patt_del_time_2 = self.mt_patt_del_time;
            self.mt_patt_del_time = 0;
        }

        // mt_dskc
        if self.mt_patt_del_time_2 != 0 {
            self.mt_patt_del_time_2 -= 1;
            if self.mt_patt_del_time_2 != 0 {
                self.mt_pattern_pos -= 1;
            }
        }

        // mt_dska
        if self.mt_pbreak_flag {
            self.mt_pbreak_flag = false;
            self.mt_pattern_pos = self.mt_pbreak_pos;
            self.mt_pbreak_pos = 0;
        }

        // mt_nnpysk
        if self.mt_pattern_pos >= 64 {
            self.mt_next_position(&module);
        }
        self.mt_no_new_pos_yet(&module);
    }

    fn mt_no_new_all_channels(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        for chn in 0..4 {
            self.mt_check_efx(chn, &mut mixer);
        }
    }

    fn mt_get_new_note(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        let pat = match module.pattern_in_position(self.mt_song_pos as usize) {
            Some(val) => val,
            None      => return,
        };

        for chn in 0..4 {
            self.mt_play_voice(pat, chn, &module, &mut mixer);
            // PT2.3D: moved from various efx
            mixer.set_volume(chn, (self.state[chn].n_volume as usize) << 4);
        }

        // mt_SetDMA
        for chn in 0..4 {
            let state = &mut self.state[chn];
            mixer.set_loop_start(chn, state.n_loopstart * 2);
            mixer.set_loop_end(chn, (state.n_loopstart + state.n_replen as u32) * 2);
            mixer.enable_loop(chn, state.n_replen > 1);
        }
    }

    fn mt_play_voice(&mut self, pat: usize, chn: usize, module: &ModData, mut mixer: &mut Mixer) {
        let event = module.patterns.event(pat, self.mt_pattern_pos, chn);

        if { let e = &self.state[chn]; e.n_note == 0 && (e.n_cmd | e.n_cmdlo == 0) } {  // TST.L   (A6)
            self.per_nop(chn, &mut mixer);
        }

        // mt_plvskip
        {
            let state = &mut self.state[chn];

            state.n_note = event.note;      // MOVE.L  (A0,D1.L),(A6)
            state.n_cmd = event.cmd;
            state.n_cmdlo = event.cmdlo;

            let ins = (((event.note & 0xf000) >> 8) | ((event.cmd as u16 & 0xf0) >> 4)) as usize;

            if ins > 0 && ins <= 31 {       // sanity check: was: ins != 0
                let instrument = &module.instruments[ins - 1];
                //state.n_reallength = instrument.size;
                // PT2.3D fix: mask finetune
                state.n_finetune = instrument.finetune & 0x0f;
                state.n_volume = instrument.volume as u8;
                state.n_length = instrument.repeat + instrument.replen;
                state.n_replen = instrument.replen;

                mixer.set_patch(chn, ins - 1, ins - 1);
                mixer.set_volume(chn, (instrument.volume as usize) << 4);  // MOVE.W  D0,8(A5)        ; Set volume
                if instrument.replen > 1 {
                    state.n_loopstart = instrument.repeat as u32;
                    state.n_wavestart = instrument.repeat as u32;
                    state.n_length = instrument.repeat + state.n_replen;
                } else {
                    // mt_NoLoop
                    state.n_length = instrument.repeat + instrument.replen;
                    state.n_replen = instrument.replen;
                }
            }
        }

        // mt_SetRegs
        if self.state[chn].n_note & 0xfff != 0 {
            match self.state[chn].n_cmd & 0x0f {
                0xe => {
                           if (self.state[chn].n_cmdlo & 0xf0) == 0x50 {
                               // mt_DoSetFinetune
                               self.mt_set_finetune(chn);
                           }
                           self.mt_set_period(chn, &mut mixer);
                       }
                0x3 => {  // TonePortamento
                           self.mt_set_tone_porta(chn);
                           self.mt_check_more_efx(chn, &mut mixer)
                       },
                0x5 => {
                           self.mt_set_tone_porta(chn);
                           self.mt_check_more_efx(chn, &mut mixer)
                       },
                0x9 => {  // Sample Offset
                           self.mt_check_more_efx(chn, &mut mixer);
                           self.mt_set_period(chn, &mut mixer);
                       },
                _   => {
                           self.mt_set_period(chn, &mut mixer);
                       },
            }
        } else {
            self.mt_check_more_efx(chn, &mut mixer);  // If no note
        }
    }

    fn mt_set_period(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let state = &mut self.state[chn];
            let note = state.n_note & 0xfff;

            let mut i = 0;      // MOVEQ   #0,D0
            // mt_ftuloop
            while i < 36 {
                if note >= MT_PERIOD_TABLE[i] {   // CMP.W   (A1,D0.W),D1
                    break;      // BHS.S   mt_ftufound
                }
                i += 1;         // ADDQ.L  #2,D0
            }                   // DBRA    D7,mt_ftuloop
            // mt_ftufound
            state.n_period = MT_PERIOD_TABLE[37 * state.n_finetune as usize + i];

            if state.n_cmd & 0x0f != 0x0e || (state.n_cmdlo & 0xf0) != 0xd0 {  // !Notedelay
                if state.n_wavecontrol & 0x04 != 0x00 {
                    state.n_vibratopos = 0;
                }
                if state.n_wavecontrol & 0x40 != 0x00 {
                    state.n_tremolopos = 0;
                }
                mixer.set_voicepos(chn, 0.0);
                mixer.set_period(chn, state.n_period as f64);
            }
        }

        self.mt_check_more_efx(chn, &mut mixer);
    }

    fn mt_next_position(&mut self, module: &ModData) {
        self.mt_pattern_pos = self.mt_pbreak_pos;
        self.mt_pbreak_pos = 0;
        self.mt_pos_jump_flag = false;
        self.mt_song_pos = self.mt_song_pos.wrapping_add(1);
        self.mt_song_pos &= 0x7f;
        if self.mt_song_pos as usize >= module.len() {
            self.mt_song_pos = 0;
        }
    }

    fn mt_no_new_pos_yet(&mut self, module: &ModData) {
        if self.mt_pos_jump_flag {
            self.mt_next_position(&module);
            self.mt_no_new_pos_yet(&module);
        }
    }

    fn mt_check_efx(&mut self, chn: usize, mut mixer: &mut Mixer) {

        let cmd = self.state[chn].n_cmd & 0x0f;

        // mt_UpdateFunk
        if cmd == 0 && self.state[chn].n_cmdlo == 0 {
            self.per_nop(chn, &mut mixer);
            return
        }

        match cmd {
            0x0 => self.mt_arpeggio(chn, &mut mixer),
            0x1 => self.mt_porta_up(chn, &mut mixer),
            0x2 => self.mt_porta_down(chn, &mut mixer),
            0x3 => self.mt_tone_portamento(chn, &mut mixer),
            0x4 => self.mt_vibrato(chn, &mut mixer),
            0x5 => self.mt_tone_plus_vol_slide(chn, &mut mixer),
            0x6 => self.mt_vibrato_plus_vol_slide(chn, &mut mixer),
            0xe => self.mt_e_commands(chn, &mut mixer),
            _   => {
                       // SetBack
                       mixer.set_period(chn, self.state[chn].n_period as f64);  // MOVE.W  n_period(A6),6(A5)
                       match cmd {
                           0x7 => self.mt_tremolo(chn, &mut mixer),
                           0xa => self.mt_volume_slide(chn),
                           _   => {},
                       }
                   }
        }
    }

    fn per_nop(&self, chn: usize, mixer: &mut Mixer) {
        let period = self.state[chn].n_period;
        mixer.set_period(chn, period as f64);  // MOVE.W  n_period(A6),6(A5)
    }

    fn mt_arpeggio(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        let val = match self.mt_counter % 3 {
            2 => {  // Arpeggio1
                     state.n_cmdlo & 15
                 },
            0 => {  // Arpeggio2
                     0
                 },
            _ => {
                     state.n_cmdlo >> 4
                 },
        } as usize;

        // Arpeggio3
        let ofs = 37 * state.n_finetune as usize;  // MOVE.B  n_finetune(A6),D1 / MULU    #36*2,D1

        // mt_arploop
        for i in 0..36 {
            if state.n_period >= MT_PERIOD_TABLE[ofs + i] {
               // Arpeggio4
               mixer.set_period(chn, MT_PERIOD_TABLE[ofs + i + val] as f64);  // MOVE.W  D2,6(A5)
               return
            }
        }
    }

    fn mt_fine_porta_up(&mut self, chn: usize, mut mixer: &mut Mixer) {
        if self.mt_counter != 0 {
            return
        }
        self.mt_low_mask = 0x0f;
        self.mt_porta_up(chn, &mut mixer);
    }

    fn mt_porta_up(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        state.n_period -= (state.n_cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if state.n_period < 113 {
            state.n_period = 113;
        }
        mixer.set_period(chn, state.n_period as f64);  // MOVE.W  n_period(A6),6(A5)
    }

    fn mt_fine_porta_down(&mut self, chn: usize, mut mixer: &mut Mixer) {
        if self.mt_counter != 0 {
            return
        }
        self.mt_low_mask = 0x0f;
        self.mt_porta_down(chn, &mut mixer);
    }

    fn mt_porta_down(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        state.n_period += (state.n_cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if state.n_period > 856 {
            state.n_period = 856;
        }
        mixer.set_period(chn, state.n_period as f64);  // MOVE.W  D0,6(A5)
    }

    fn mt_set_tone_porta(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        let note = state.n_note & 0xfff;
        let ofs = 37 * state.n_finetune as usize;  // MOVE.B  n_finetune(A6),D0 / MULU    #37*2,D0

        let mut i = 0;       // MOVEQ   #0,D0
        // mt_StpLoop
        while note < MT_PERIOD_TABLE[ofs + i] {    // BHS.S   mt_StpFound
            i += 1;          // ADDQ.W  #2,D0
            if i >= 37 {     // CMP.W   #37*2,D0 / BLO.S   mt_StpLoop
                i = 35;      // MOVEQ   #35*2,D0
                break
            }
        }

        // mt_StpFound
        if state.n_finetune & 0x80 != 0 && i != 0 {
            i -= 1;          // SUBQ.W  #2,D0
        }
        // mt_StpGoss
        state.n_wantedperiod = MT_PERIOD_TABLE[ofs + i];
        state.n_toneportdirec = false;

        if state.n_period == state.n_wantedperiod {
            // mt_ClearTonePorta
            state.n_wantedperiod = 0;
        } else if state.n_period < state.n_wantedperiod {
            state.n_toneportdirec = true;
        }
    }

    fn mt_tone_portamento(&mut self, chn: usize, mut mixer: &mut Mixer) {
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
            return
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
        let mut period = state.n_period;               // MOVE.W  n_period(A6),D2
        if state.n_glissfunk & 0x0f != 0 {
            let ofs = 37 * state.n_finetune as usize;  // MULU    #36*2,D0
            let mut i = 0;
            // mt_GlissLoop
            while period < MT_PERIOD_TABLE[ofs + i] {  // LEA     mt_PeriodTable(PC),A0 / CMP.W   (A0,D0.W),D2
                i += 1;
                if i >= 37 {
                    i = 35;
                    break;
                }
            }
            // mt_GlissFound
            period = MT_PERIOD_TABLE[ofs + i];         // MOVE.W  (A0,D0.W),D2
        }
        // mt_GlissSkip
        mixer.set_period(chn, period as f64);          // MOVE.W  D2,6(A5) ; Set period
    }

    fn mt_vibrato(&mut self, chn: usize, mut mixer: &mut Mixer) {
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
        self.mt_vibrato_2(chn, &mut mixer);
    }

    fn mt_vibrato_2(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        let mut pos = (state.n_vibratopos >> 2) & 0x1f;
        let val = match state.n_wavecontrol & 0x03 {
            0 => {  // mt_vib_sine
                     MT_VIBRATO_TABLE[pos as usize]
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

    fn mt_tone_plus_vol_slide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        self.mt_tone_port_no_change(chn, &mut mixer);
        self.mt_volume_slide(chn);
    }

    fn mt_vibrato_plus_vol_slide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        self.mt_vibrato_2(chn, &mut mixer);
        self.mt_volume_slide(chn);
    }

    fn mt_tremolo(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if state.n_cmdlo != 0 {
            if state.n_cmdlo & 0x0f != 0 {
                 state.n_tremolocmd = (state.n_cmdlo & 0x0f) | (state.n_tremolocmd & 0xf0)
            }
            // mt_treskip
            if state.n_cmdlo & 0xf0 != 0 {
                 state.n_tremolocmd = (state.n_cmdlo & 0xf0) | (state.n_tremolocmd & 0x0f)
            }
            // mt_treskip2
        }
        // mt_Tremolo2
        let mut pos = (state.n_tremolopos >> 2) & 0x1f;
        let val = match (state.n_wavecontrol >> 4) & 0x03 {
            0 => {  // mt_tre_sine
                     MT_VIBRATO_TABLE[pos as usize]
                 },
            1 => {  // mt_rampdown
                     pos <<= 3;
                     if pos & 0x80 != 0 { 255 - pos } else { pos }
                 },
            _ => {
                     255
                 },
        };
        // mt_tre_set
        let mut volume = state.n_volume as isize;
        let amt = ((val as usize * (state.n_tremolocmd & 15) as usize) >> 6) as isize;
        if state.n_tremolopos & 0x80 == 0 {
            volume += amt;
        } else {
            volume -= amt;
        }
        // mt_Tremolo3
        if volume < 0 {
            volume = 0;
        }
        // mt_TremoloSkip
        if volume > 0x40 {
           volume = 0x40;
        }

        // mt_TremoloOk
        mixer.set_volume(chn, (volume as usize) << 4);  // MOVE.W  D0,8(A5)
        state.n_tremolopos = state.n_tremolopos.wrapping_add((state.n_tremolocmd >> 2) & 0x3c);
    }

    fn mt_sample_offset(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if state.n_cmdlo != 0 {
            state.n_sampleoffset = state.n_cmdlo;
        }
        mixer.set_voicepos(chn, ((state.n_sampleoffset as u32) << 8) as f64);
    }

    fn mt_volume_slide(&mut self, chn: usize) {
        if self.state[chn].n_cmdlo >> 4 == 0 {
            self.mt_vol_slide_down(chn);
        } else {
            self.mt_vol_slide_up(chn);
        }
    }

    fn mt_vol_slide_up(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        state.n_volume += state.n_cmdlo >> 4;
        if state.n_volume > 0x40 {
            state.n_volume = 0x40;
        }
        // PT2.3D: moved to mt_GetNewNote
        //mixer.set_volume(chn, (state.n_volume as usize) << 4);
    }

    fn mt_vol_slide_down(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        let cmdlo = state.n_cmdlo & 0x0f;
        if state.n_volume > cmdlo {
            state.n_volume -= cmdlo;
        } else {
            state.n_volume = 0;
        }
        // PT2.3D: moved to mt_GetNewNote
        //mixer.set_volume(chn, (state.n_volume as usize) << 4);
    }

    fn mt_position_jump(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        self.mt_song_pos = state.n_cmdlo.wrapping_sub(1);
        // mt_pj2
        self.mt_pbreak_pos = 0;
        self.mt_pos_jump_flag = true;
    }

    fn mt_volume_change(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        if state.n_cmdlo > 0x40 {
            state.n_cmdlo = 40
        }
        state.n_volume = state.n_cmdlo;
        // PT2.3D: moved to mt_GetNewNote
        //mixer.set_volume(chn, (state.n_volume as usize) << 4);  // MOVE.W  D0,8(A5)
    }

    fn mt_pattern_break(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        let row = (state.n_cmdlo >> 4) * 10 + (state.n_cmdlo & 0x0f);
        if row <= 63 {
            // mt_pj2
            self.mt_pbreak_pos = row;
        }
        self.mt_pos_jump_flag = true;
    }

    fn mt_set_speed(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        if state.n_cmdlo != 0 {
            self.mt_counter = 0;
            // also check CIA tempo
            if state.n_cmdlo < 0x20 {
                self.mt_speed = state.n_cmdlo;
            } else {
                self.cia_tempo = state.n_cmdlo;
            }
        }
    }

    fn mt_check_more_efx(&mut self, chn: usize, mut mixer: &mut Mixer) {
        // mt_UpdateFunk()

        match self.state[chn].n_cmd & 0x0f {
            0x9 => self.mt_sample_offset(chn, &mut mixer),
            0xb => self.mt_position_jump(chn),
            0xd => self.mt_pattern_break(chn),
            0xe => self.mt_e_commands(chn, &mut mixer),
            0xf => self.mt_set_speed(chn),
            0xc => self.mt_volume_change(chn),
            _   => {},
        }

        self.per_nop(chn, &mut mixer)
    }

    fn mt_e_commands(&mut self, chn: usize, mut mixer: &mut Mixer) {

        match self.state[chn].n_cmdlo >> 4 {
           0x0 => self.mt_filter_on_off(),
           0x1 => self.mt_fine_porta_up(chn, &mut mixer),
           0x2 => self.mt_fine_porta_down(chn, &mut mixer),
           0x3 => self.mt_set_gliss_control(chn),
           0x4 => self.mt_set_vibrato_control(chn),
           0x5 => self.mt_set_finetune(chn),
           0x6 => self.mt_jump_loop(chn),
           0x7 => self.mt_set_tremolo_control(chn),
           0x9 => self.mt_retrig_note(chn, &mut mixer),
           0xa => self.mt_volume_fine_up(chn),
           0xb => self.mt_volume_fine_down(chn),
           0xc => self.mt_note_cut(chn, &mut mixer),
           0xd => self.mt_note_delay(chn, &mut mixer),
           0xe => self.mt_pattern_delay(chn),
           0xf => self.mt_funk_it(chn, &mut mixer),
           _   => {},
        }
    }

    fn mt_filter_on_off(&self) {
    }

    fn mt_set_gliss_control(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        state.n_glissfunk = state.n_cmdlo;
    }

    fn mt_set_vibrato_control(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        state.n_wavecontrol &= 0xf0;
        state.n_wavecontrol |= state.n_cmdlo & 0x0f;
    }

    fn mt_set_finetune(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        state.n_finetune = state.n_cmdlo & 0x0f;
    }

    fn mt_jump_loop(&mut self, chn: usize) {
        let state = &mut self.state[chn];

        if self.mt_counter != 0 {
            return
        }

        let cmdlo = state.n_cmdlo & 0x0f;

        if cmdlo == 0 {
            // mt_SetLoop
            state.n_pattpos = self.mt_pattern_pos as u8;
        } else {
            if state.n_loopcount == 0 {
                // mt_jmpcnt
                state.n_loopcount = cmdlo;
            } else {
                state.n_loopcount -= 1;
                if state.n_loopcount == 0 {
                    return
                }
            }
            // mt_jmploop
            self.mt_pbreak_pos = state.n_pattpos;
            self.mt_pbreak_flag = true;
        }
    }

    fn mt_set_tremolo_control(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        state.n_wavecontrol &= 0x0f;
        state.n_wavecontrol |= (state.n_cmdlo & 0x0f) << 4;
    }

    fn mt_retrig_note(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        let cmdlo = state.n_cmdlo & 0x0f;
        if cmdlo == 0 {
            return
        }
        if self.mt_counter == 0 {
            if state.n_note & 0xfff != 0 {
                return
            }
        }
        // mt_rtnskp
        if self.mt_counter % cmdlo != 0 {
            return
        }
        
        // mt_DoRetrig
        mixer.set_voicepos(chn, 0.0);
    }

    fn mt_volume_fine_up(&mut self, chn: usize) {
        if self.mt_counter != 0 {
            return
        }
        self.mt_vol_slide_up(chn);
    }

    fn mt_volume_fine_down(&mut self, chn: usize) {
        if self.mt_counter != 0 {
            return
        }
        self.mt_vol_slide_down(chn)
    }

    fn mt_note_cut(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if self.mt_counter != state.n_cmdlo {
            return
        }
        state.n_volume = 0;
        mixer.set_volume(chn, 0);  // MOVE.W  #0,8(A5)
    }

    fn mt_note_delay(&mut self, chn: usize, mixer: &mut Mixer) {
        let state = &mut self.state[chn];
        if self.mt_counter != state.n_cmdlo {
            return
        }
        // PT2.3D/E fix: mask note
        if state.n_note & 0xfff == 0 {
            return
        }
        // BRA mt_DoRetrig
        mixer.set_voicepos(chn, 0.0);
    }

    fn mt_pattern_delay(&mut self, chn: usize) {
        let state = &mut self.state[chn];
        if self.mt_counter != 0 {
            return
        }
        if self.mt_patt_del_time_2 != 0 {
            return
        }
        self.mt_patt_del_time = state.n_cmdlo & 0x0f + 1;
    }

    fn mt_funk_it(&self, _chn: usize, _mixer: &mut Mixer) {
    }
}


/*
static MT_FUNK_TABLE: [u8; 16] = [
    0, 5, 6, 7, 8, 10, 11, 13, 16, 19, 22, 26, 32, 43, 64, 128
];
*/

static MT_VIBRATO_TABLE: [u8; 32] = [
      0,  24,  49,  74,  97, 120, 141, 161,
    180, 197, 212, 224, 235, 244, 250, 253,
    255, 253, 250, 244, 235, 224, 212, 197,
    180, 161, 141, 120,  97,  74,  49,  24
];


// PT2.3D fix: add trailing zeros
static MT_PERIOD_TABLE: [u16; 16*37] = [
// Tuning 0, Normal
    856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
    428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
    214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113, 0,
// Tuning 1
    850, 802, 757, 715, 674, 637, 601, 567, 535, 505, 477, 450,
    425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 239, 225,
    213, 201, 189, 179, 169, 159, 150, 142, 134, 126, 119, 113, 0,
// Tuning 2
    844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474, 447,
    422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237, 224,
    211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118, 112, 0,
// Tuning 3
    838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470, 444,
    419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235, 222,
    209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118, 111, 0,
// Tuning 4
    832, 785, 741, 699, 660, 623, 588, 555, 524, 495, 467, 441,
    416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233, 220,
    208, 196, 185, 175, 165, 156, 147, 139, 131, 124, 117, 110, 0,
// Tuning 5
    826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463, 437,
    413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232, 219,
    206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116, 109, 0,
// Tuning 6
    820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460, 434,
    410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230, 217,
    205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115, 109, 0,
// Tuning 7
    814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457, 431,
    407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228, 216,
    204, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114, 108, 0,
// Tuning -8
    907, 856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480,
    453, 428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240,
    226, 214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 0,
// Tuning -7
    900, 850, 802, 757, 715, 675, 636, 601, 567, 535, 505, 477,
    450, 425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 238,
    225, 212, 200, 189, 179, 169, 159, 150, 142, 134, 126, 119, 0,
// Tuning -6
    894, 844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474,
    447, 422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237,
    223, 211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118, 0,
// Tuning -5
    887, 838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470,
    444, 419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235,
    222, 209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118, 0,
// Tuning -4
    881, 832, 785, 741, 699, 660, 623, 588, 555, 524, 494, 467,
    441, 416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233,
    220, 208, 196, 185, 175, 165, 156, 147, 139, 131, 123, 117, 0,
// Tuning -3
    875, 826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463,
    437, 413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232,
    219, 206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116, 0,
// Tuning -2
    868, 820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460,
    434, 410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230,
    217, 205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115, 0,
// Tuning -1
    862, 814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457,
    431, 407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228,
    216, 203, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114, 0
];


#[derive(Clone,Default)]
struct ChannelData {
    n_note         : u16,
    n_cmd          : u8,
    n_cmdlo        : u8,
    n_length       : u16,
    n_loopstart    : u32,
    n_replen       : u16,
    n_period       : u16,
    n_finetune     : u8,
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

        self.mt_song_pos = data.pos as u8;
        self.mt_pattern_pos = data.row as u8;
        self.mt_counter = data.frame as u8;

        self.mt_music(&module, &mut mixer);

        data.frame = self.mt_counter as usize;
        data.row = self.mt_pattern_pos as usize;
        data.pos = self.mt_song_pos as usize;
        data.speed = self.mt_speed as usize;
        data.tempo = self.cia_tempo as usize;
    }

    fn reset(&mut self) {
        self.mt_speed           = 6;
        self.mt_counter         = 0;
        self.mt_song_pos        = 0;
        self.mt_pbreak_pos      = 0;
        self.mt_pos_jump_flag   = false;
        self.mt_pbreak_flag     = false;
        self.mt_low_mask        = 0;
        self.mt_patt_del_time   = 0;
        self.mt_patt_del_time_2 = 0;
        self.mt_pattern_pos     = 0;
    }
}
