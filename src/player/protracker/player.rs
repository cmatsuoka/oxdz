use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer, State};
use player::scan::SaveRestore;
use format::mk::ModData;
use mixer::Mixer;

/// PT2.1A Replayer
///
/// An oxdz player based on the Protracker V2.1A replayer written by Peter "CRAYON"
/// Hanning / Mushroom Studios in 1992. Original names are used whenever possible
/// (converted to snake case according to Rust convention, i.e. mt_PosJumpFlag
/// becomes mt_pos_jump_flag).
///
/// Bug fixes backported from Protracker 2.3D:
/// * Mask finetune when playing voice
/// * Mask note value in note delay command processing
/// * Fix period table lookup by adding trailing zero values

#[derive(SaveRestore)]
pub struct ModPlayer {
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

    mt_chantemp       : [ChannelData; 4],
    mt_samplestarts   : [u32; 31],
}

impl ModPlayer {
    pub fn new(_module: &Module, options: Options) -> Self {
        ModPlayer {
            options,

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

            mt_chantemp       : [ChannelData::new(); 4],
            mt_samplestarts   : [0; 31],
        }
    }

    fn mt_music(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        self.mt_counter += 1;
        if self.mt_speed > self.mt_counter {
            // mt_NoNewNote
            self.mt_no_new_all_channels(&mut mixer);
            self.mt_no_new_pos_yet(&module);
            return
        }

        self.mt_counter = 0;
        if self.mt_patt_del_time_2 == 0 {
            self.mt_get_new_note(&module, &mut mixer);
        } else {
            self.mt_no_new_all_channels(&mut mixer);
        }

        // mt_dskip
        self.mt_pattern_pos += 1;
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

    fn mt_no_new_all_channels(&mut self, mut mixer: &mut Mixer) {
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
        }

        // mt_SetDMA
        for chn in 0..4 {
            let ch = &mut self.mt_chantemp[chn];
            mixer.set_loop_start(chn, ch.n_loopstart - ch.n_start);
            mixer.set_loop_end(chn, ch.n_loopstart - ch.n_start + ch.n_replen as u32 * 2);
            mixer.enable_loop(chn, ch.n_replen > 1);
        }
    }

    fn mt_play_voice(&mut self, pat: usize, chn: usize, module: &ModData, mut mixer: &mut Mixer) {
        let event = module.patterns.event(pat, self.mt_pattern_pos, chn);

        if { let e = &self.mt_chantemp[chn]; e.n_note == 0 && (e.n_cmd | e.n_cmdlo == 0) } {  // TST.L   (A6)
            self.per_nop(chn, &mut mixer);
        }

        // mt_plvskip
        {
            let ch = &mut self.mt_chantemp[chn];

            ch.n_note = event.note;      // MOVE.L  (A0,D1.L),(A6)
            ch.n_cmd = event.cmd;
            ch.n_cmdlo = event.cmdlo;

            let ins = (((event.note & 0xf000) >> 8) | ((event.cmd as u16 & 0xf0) >> 4)) as usize;

            if ins > 0 && ins <= 31 {       // sanity check: was: ins != 0
                let instrument = &module.instruments[ins - 1];
                ch.n_start = self.mt_samplestarts[ins - 1];
                ch.n_length = instrument.size;
                //ch.n_reallength = instrument.size;
                // PT2.3D fix: mask finetune
                ch.n_finetune = instrument.finetune & 0x0f;
                ch.n_volume = instrument.volume;
                ch.n_replen = instrument.replen;

                if instrument.repeat != 0 {
                    ch.n_loopstart = ch.n_start + instrument.repeat as u32 * 2;
                    ch.n_wavestart = ch.n_loopstart;
                    ch.n_length = (instrument.repeat + ch.n_replen) * 2;
                    mixer.set_volume(chn, (instrument.volume as usize) << 4);  // MOVE.W  D0,8(A5)        ; Set volume
                } else {
                    // mt_NoLoop
                    ch.n_loopstart = ch.n_start;
                    ch.n_wavestart = ch.n_start;
                    ch.n_replen = instrument.replen;                           // MOVE.W  6(A3,D4.L),n_replen(A6) ; Save replen
                    mixer.set_volume(chn, (instrument.volume as usize) << 4);  // MOVE.W  D0,8(A5)        ; Set volume
                }
            }
        }

        // mt_SetRegs
        if self.mt_chantemp[chn].n_note & 0xfff != 0 {
            match self.mt_chantemp[chn].n_cmd & 0x0f {
                0xe =>  {
                           if (self.mt_chantemp[chn].n_cmdlo & 0xf0) == 0x50 {
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
            let ch = &mut self.mt_chantemp[chn];
            let note = ch.n_note & 0xfff;

            let mut i = 0;                          // MOVEQ   #0,D0
            // mt_ftuloop
            while i < 36 {
                if note >= MT_PERIOD_TABLE[i] {     // CMP.W   (A1,D0.W),D1
                    break;                          // BHS.S   mt_ftufound
                }
                i += 1;                             // ADDQ.L  #2,D0
            }                                       // DBRA    D7,mt_ftuloop
            // mt_ftufound
            ch.n_period = MT_PERIOD_TABLE[37 * ch.n_finetune as usize + i];

            if ch.n_cmd & 0x0f != 0x0e || (ch.n_cmdlo & 0xf0) != 0xd0 {  // !Notedelay
                if ch.n_wavecontrol & 0x04 != 0x00 {
                    ch.n_vibratopos = 0;
                }
                // mt_vibnoc
                if ch.n_wavecontrol & 0x40 != 0x00 {
                    ch.n_tremolopos = 0;
                }
                // mt_trenoc
                mixer.set_sample_ptr(chn, ch.n_start);
                mixer.set_period(chn, ch.n_period as f64);
                //mixer.set_voicepos(chn, 0.0);
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
        if self.mt_song_pos >= module.song_length {
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

        let cmd = self.mt_chantemp[chn].n_cmd & 0x0f;

        // mt_UpdateFunk
        if cmd == 0 && self.mt_chantemp[chn].n_cmdlo == 0 {
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
                mixer.set_period(chn, self.mt_chantemp[chn].n_period as f64);  // MOVE.W  n_period(A6),6(A5)
                match cmd {
                    0x7 => self.mt_tremolo(chn, &mut mixer),
                    0xa => self.mt_volume_slide(chn, &mut mixer),
                    _   => (),
                }
            }
        }

        if cmd != 0x7 {
            mixer.set_volume(chn, (self.mt_chantemp[chn].n_volume as usize) << 4);
        }
    }

    fn per_nop(&self, chn: usize, mixer: &mut Mixer) {
        let period = self.mt_chantemp[chn].n_period;
        mixer.set_period(chn, period as f64);  // MOVE.W  n_period(A6),6(A5)
    }

    fn mt_arpeggio(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        let val = match self.mt_counter % 3 {
            2 => ch.n_cmdlo & 15,  // Arpeggio1
            0 => 0,                // Arpeggio2
            _ => ch.n_cmdlo >> 4,
        } as usize;

        // Arpeggio3
        let ofs = 37 * ch.n_finetune as usize;  // MOVE.B  n_finetune(A6),D1 / MULU    #36*2,D1

        // mt_arploop
        for i in 0..36 {
            if ch.n_period >= MT_PERIOD_TABLE[ofs + i] {
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
        let ch = &mut self.mt_chantemp[chn];
        ch.n_period -= (ch.n_cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if ch.n_period < 113 {
            ch.n_period = 113;
        }
        mixer.set_period(chn, ch.n_period as f64);  // MOVE.W  n_period(A6),6(A5)
    }

    fn mt_fine_porta_down(&mut self, chn: usize, mut mixer: &mut Mixer) {
        if self.mt_counter != 0 {
            return
        }
        self.mt_low_mask = 0x0f;
        self.mt_porta_down(chn, &mut mixer);
    }

    fn mt_porta_down(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        ch.n_period += (ch.n_cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if ch.n_period > 856 {
            ch.n_period = 856;
        }
        mixer.set_period(chn, ch.n_period as f64);  // MOVE.W  D0,6(A5)
    }

    fn mt_set_tone_porta(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        let note = ch.n_note & 0xfff;
        let ofs = 37 * ch.n_finetune as usize;  // MOVE.B  n_finetune(A6),D0 / MULU    #37*2,D0

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
        if ch.n_finetune & 0x80 != 0 && i != 0 {
            i -= 1;          // SUBQ.W  #2,D0
        }
        // mt_StpGoss
        ch.n_wantedperiod = MT_PERIOD_TABLE[ofs + i];
        ch.n_toneportdirec = false;

        if ch.n_period == ch.n_wantedperiod {
            // mt_ClearTonePorta
            ch.n_wantedperiod = 0;
        } else if ch.n_period < ch.n_wantedperiod {
            ch.n_toneportdirec = true;
        }
    }

    fn mt_tone_portamento(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.mt_chantemp[chn];
            if ch.n_cmdlo != 0 {
                ch.n_toneportspeed = ch.n_cmdlo;
                ch.n_cmdlo = 0;
            }
        }
        self.mt_tone_port_no_change(chn, &mut mixer);
    }

    fn mt_tone_port_no_change(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        if ch.n_wantedperiod == 0 {
            return
        }
        if ch.n_toneportdirec {
            // mt_TonePortaDown
            ch.n_period += ch.n_toneportspeed as u16;
            if ch.n_period > ch.n_wantedperiod {
                ch.n_period = ch.n_wantedperiod;
                ch.n_wantedperiod = 0;
            }
        } else {
            // mt_TonePortaUp
            if ch.n_period > ch.n_toneportspeed as u16 {
                ch.n_period -= ch.n_toneportspeed as u16;
            } else {
                ch.n_period = 0;
            }
            if ch.n_period < ch.n_wantedperiod {
                ch.n_period = ch.n_wantedperiod;
                ch.n_wantedperiod = 0;
            }
        }
        // mt_TonePortaSetPer
        let mut period = ch.n_period;                   // MOVE.W  n_period(A6),D2
        if ch.n_glissfunk & 0x0f != 0 {
            let ofs = 37 * ch.n_finetune as usize;      // MULU    #36*2,D0
            let mut i = 0;
            // mt_GlissLoop
            while period < MT_PERIOD_TABLE[ofs + i] {   // LEA     mt_PeriodTable(PC),A0 / CMP.W   (A0,D0.W),D2
                i += 1;
                if i >= 37 {
                    i = 35;
                    break;
                }
            }
            // mt_GlissFound
            period = MT_PERIOD_TABLE[ofs + i];          // MOVE.W  (A0,D0.W),D2
        }
        // mt_GlissSkip
        mixer.set_period(chn, period as f64);           // MOVE.W  D2,6(A5) ; Set period
    }

    fn mt_vibrato(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.mt_chantemp[chn];
            if ch.n_cmdlo != 0 {
                if ch.n_cmdlo & 0x0f != 0 {
                    ch.n_vibratocmd = (ch.n_vibratocmd & 0xf0) | (ch.n_cmdlo & 0x0f)
                }
                // mt_vibskip
                if ch.n_cmdlo & 0xf0 != 0 {
                    ch.n_vibratocmd = (ch.n_vibratocmd & 0x0f) | (ch.n_cmdlo & 0xf0)
                }
                // mt_vibskip2
            }
        }
        self.mt_vibrato_2(chn, &mut mixer);
    }

    fn mt_vibrato_2(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        let mut pos = (ch.n_vibratopos >> 2) & 0x1f;
        let val = match ch.n_wavecontrol & 0x03 {
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
        let mut period = ch.n_period;
        let amt = (val as usize * (ch.n_vibratocmd & 15) as usize) >> 7;
        if ch.n_vibratopos & 0x80 == 0 {
            period += amt as u16
        } else {
            period -= amt as u16
        };

        // mt_Vibrato3
        mixer.set_period(chn, period as f64);
        ch.n_vibratopos = ch.n_vibratopos.wrapping_add((ch.n_vibratocmd >> 2) & 0x3c);
    }

    fn mt_tone_plus_vol_slide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        self.mt_tone_port_no_change(chn, &mut mixer);
        self.mt_volume_slide(chn, &mut mixer);
    }

    fn mt_vibrato_plus_vol_slide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        self.mt_vibrato_2(chn, &mut mixer);
        self.mt_volume_slide(chn, &mut mixer);
    }

    fn mt_tremolo(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        if ch.n_cmdlo != 0 {
            if ch.n_cmdlo & 0x0f != 0 {
                 ch.n_tremolocmd = (ch.n_cmdlo & 0x0f) | (ch.n_tremolocmd & 0xf0)
            }
            // mt_treskip
            if ch.n_cmdlo & 0xf0 != 0 {
                 ch.n_tremolocmd = (ch.n_cmdlo & 0xf0) | (ch.n_tremolocmd & 0x0f)
            }
            // mt_treskip2
        }
        // mt_Tremolo2
        let mut pos = (ch.n_tremolopos >> 2) & 0x1f;
        let val = match (ch.n_wavecontrol >> 4) & 0x03 {
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
        let mut volume = ch.n_volume as isize;
        let amt = ((val as usize * (ch.n_tremolocmd & 15) as usize) >> 6) as isize;
        if ch.n_tremolopos & 0x80 == 0 {
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
        ch.n_tremolopos = ch.n_tremolopos.wrapping_add((ch.n_tremolocmd >> 2) & 0x3c);
    }

    fn mt_sample_offset(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        if ch.n_cmdlo != 0 {
            ch.n_sampleoffset = ch.n_cmdlo;
        }
        mixer.set_voicepos(chn, ((ch.n_sampleoffset as u32) << 8) as f64);
    }

    fn mt_volume_slide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        let val = self.mt_chantemp[chn].n_cmdlo >> 4;
        if val == 0 {
            self.mt_vol_slide_down(chn, &mut mixer);
        } else {
            self.mt_vol_slide_up(chn, val, &mut mixer);
        }
    }

    fn mt_vol_slide_up(&mut self, chn: usize, val: u8, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        ch.n_volume += val;
        if ch.n_volume > 0x40 {
            ch.n_volume = 0x40;
        }
        mixer.set_volume(chn, (ch.n_volume as usize) << 4);
    }

    fn mt_vol_slide_down(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        let val = ch.n_cmdlo & 0x0f;
        if ch.n_volume > val {
            ch.n_volume -= val;
        } else {
            ch.n_volume = 0;
        }
        mixer.set_volume(chn, (ch.n_volume as usize) << 4);
    }

    fn mt_position_jump(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        self.mt_song_pos = ch.n_cmdlo.wrapping_sub(1);
        // mt_pj2
        self.mt_pbreak_pos = 0;
        self.mt_pos_jump_flag = true;
    }

    fn mt_volume_change(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        if ch.n_cmdlo > 0x40 {
            ch.n_cmdlo = 0x40
        }
        ch.n_volume = ch.n_cmdlo;
        mixer.set_volume(chn, (ch.n_volume as usize) << 4);  // MOVE.W  D0,8(A5)
    }

    fn mt_pattern_break(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        let row = (ch.n_cmdlo >> 4) * 10 + (ch.n_cmdlo & 0x0f);
        if row <= 63 {
            // mt_pj2
            self.mt_pbreak_pos = row;
        }
        self.mt_pos_jump_flag = true;
    }

    fn mt_set_speed(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        if ch.n_cmdlo != 0 {
            self.mt_counter = 0;
            // also check CIA tempo
            if ch.n_cmdlo < 0x20 {
                self.mt_speed = ch.n_cmdlo;
            } else {
                self.cia_tempo = ch.n_cmdlo;
            }
        }
    }

    fn mt_check_more_efx(&mut self, chn: usize, mut mixer: &mut Mixer) {
        // mt_UpdateFunk()

        match self.mt_chantemp[chn].n_cmd & 0x0f {
            0x9 => self.mt_sample_offset(chn, &mut mixer),
            0xb => self.mt_position_jump(chn),
            0xd => self.mt_pattern_break(chn),
            0xe => self.mt_e_commands(chn, &mut mixer),
            0xf => self.mt_set_speed(chn),
            0xc => self.mt_volume_change(chn, &mut mixer),
            _   => {},
        }

        self.per_nop(chn, &mut mixer)
    }

    fn mt_e_commands(&mut self, chn: usize, mut mixer: &mut Mixer) {

        match self.mt_chantemp[chn].n_cmdlo >> 4 {
            0x0 => self.mt_filter_on_off(chn, &mut mixer),
            0x1 => self.mt_fine_porta_up(chn, &mut mixer),
            0x2 => self.mt_fine_porta_down(chn, &mut mixer),
            0x3 => self.mt_set_gliss_control(chn),
            0x4 => self.mt_set_vibrato_control(chn),
            0x5 => self.mt_set_finetune(chn),
            0x6 => self.mt_jump_loop(chn),
            0x7 => self.mt_set_tremolo_control(chn),
            0x9 => self.mt_retrig_note(chn, &mut mixer),
            0xa => self.mt_volume_fine_up(chn, &mut mixer),
            0xb => self.mt_volume_fine_down(chn, &mut mixer),
            0xc => self.mt_note_cut(chn, &mut mixer),
            0xd => self.mt_note_delay(chn, &mut mixer),
            0xe => self.mt_pattern_delay(chn),
            0xf => self.mt_funk_it(chn, &mut mixer),
            _   => {},
        }
    }

    fn mt_filter_on_off(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        mixer.enable_filter(ch.n_cmdlo & 0x0f != 0);
    }

    fn mt_set_gliss_control(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        ch.n_glissfunk = (ch.n_glissfunk & 0xf0) | (ch.n_cmdlo & 0x0f);
    }

    fn mt_set_vibrato_control(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        ch.n_wavecontrol &= 0xf0;
        ch.n_wavecontrol |= ch.n_cmdlo & 0x0f;
    }

    fn mt_set_finetune(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        ch.n_finetune = ch.n_cmdlo & 0x0f;
    }

    fn mt_jump_loop(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];

        if self.mt_counter != 0 {
            return
        }

        let cmdlo = ch.n_cmdlo & 0x0f;

        if cmdlo == 0 {
            // mt_SetLoop
            ch.n_pattpos = self.mt_pattern_pos as u8;
        } else {
            if ch.n_loopcount == 0 {
                // mt_jmpcnt
                ch.n_loopcount = cmdlo;
                ch.inside_loop = true;
            } else {
                ch.n_loopcount -= 1;
                if ch.n_loopcount == 0 {
                    ch.inside_loop = false;
                    return
                }
            }
            // mt_jmploop
            self.mt_pbreak_pos = ch.n_pattpos;
            self.mt_pbreak_flag = true;
        }
    }

    fn mt_set_tremolo_control(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        ch.n_wavecontrol &= 0x0f;
        ch.n_wavecontrol |= (ch.n_cmdlo & 0x0f) << 4;
    }

    fn mt_retrig_note(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        let cmdlo = ch.n_cmdlo & 0x0f;
        if cmdlo == 0 {
            return
        }
        if self.mt_counter == 0 {
            if ch.n_note & 0xfff != 0 {
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

    fn mt_volume_fine_up(&mut self, chn: usize, mut mixer: &mut Mixer) {
        if self.mt_counter != 0 {
            return
        }
        let val = self.mt_chantemp[chn].n_cmdlo & 0x0f;
        self.mt_vol_slide_up(chn, val, &mut mixer);
    }

    fn mt_volume_fine_down(&mut self, chn: usize, mut mixer: &mut Mixer) {
        if self.mt_counter != 0 {
            return
        }
        self.mt_vol_slide_down(chn, &mut mixer)
    }

    fn mt_note_cut(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        if self.mt_counter != ch.n_cmdlo {
            return
        }
        ch.n_volume = 0;
        mixer.set_volume(chn, 0);  // MOVE.W  #0,8(A5)
    }

    fn mt_note_delay(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.mt_chantemp[chn];
        if self.mt_counter != ch.n_cmdlo {
            return
        }
        // PT2.3D fix: mask note
        if ch.n_note & 0xfff == 0 {
            return
        }
        // BRA mt_DoRetrig
        mixer.set_voicepos(chn, 0.0);
    }

    fn mt_pattern_delay(&mut self, chn: usize) {
        let ch = &mut self.mt_chantemp[chn];
        if self.mt_counter != 0 {
            return
        }
        if self.mt_patt_del_time_2 != 0 {
            return
        }
        self.mt_patt_del_time = (ch.n_cmdlo & 0x0f) + 1;
    }

    fn mt_funk_it(&self, _chn: usize, _mixer: &mut Mixer) {
    }
}


/*
static MT_FUNK_TABLE: [u8; 16] = [
    0, 5, 6, 7, 8, 10, 11, 13, 16, 19, 22, 26, 32, 43, 64, 128
];
*/

lazy_static! {
    static ref MT_VIBRATO_TABLE: Box<[u8; 32]> = Box::new([
          0,  24,  49,  74,  97, 120, 141, 161,
        180, 197, 212, 224, 235, 244, 250, 253,
        255, 253, 250, 244, 235, 224, 212, 197,
        180, 161, 141, 120,  97,  74,  49,  24
    ]);

    // PT2.3D fix: add trailing zeros
    static ref MT_PERIOD_TABLE: Box<[u16; 16*37]> = Box::new([
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
    ]);
}


#[derive(Clone,Copy,Default)]
struct ChannelData {
    n_note         : u16,
    n_cmd          : u8,
    n_cmdlo        : u8,
    n_start        : u32,
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
    //n_funkoffset   : u8,
    n_wavestart    : u32,

    inside_loop    : bool,
}

impl ChannelData {
    pub fn new() -> Self {
        Default::default()
    }
}


impl FormatPlayer for ModPlayer {
    fn start(&mut self, data: &mut PlayerData, mdata: &ModuleData, mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<ModData>().unwrap();

        data.speed = 6;
        data.tempo = 125.0;
        data.time  = 0.0;

        for i in 0..31 {
            self.mt_samplestarts[i] = module.samples[i].address;
        }

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

        let module = mdata.as_any().downcast_ref::<ModData>().unwrap();

        self.mt_music(&module, &mut mixer);

        data.frame = self.mt_counter as usize;
        data.row = self.mt_pattern_pos as usize;
        data.pos = self.mt_song_pos as usize;
        data.speed = self.mt_speed as usize;
        data.tempo = self.cia_tempo as f32;
        data.time += 20.0 * 125.0 / data.tempo;

        data.inside_loop = false;
        for chn in 0..4 {
            data.inside_loop |= self.mt_chantemp[chn].inside_loop;
        }
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

    unsafe fn save_state(&self) -> State {
        self.save()
    }

    unsafe fn restore_state(&mut self, state: &State) {
        self.restore(&state)
    }
}
