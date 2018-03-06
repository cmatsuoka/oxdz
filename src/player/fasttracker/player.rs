use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer};
use format::mk::ModData;
use mixer::Mixer;

/// FT101 Replayer
///
/// An oxdz player based on the FastTracker 1.01 replayer written by Fredrik
/// Huss (Mr.H / Triton) in 1992-1993. Function and variable names from the
/// corresponding parts in the Protracker 2.1A playroutine.

pub struct FtPlayer {
    options: Options, 

    ft_speed          : u8,
    ft_counter        : u8,
    ft_song_pos       : u8,
    ft_pbreak_pos     : u8,
    ft_pos_jump_flag  : bool,
    ft_pbreak_flag    : bool,
    ft_patt_del_time  : u8,
    ft_patt_del_time_2: u8,
    ft_pattern_pos    : u8,
    cia_tempo         : u8,

    ft_chantemp       : Vec<ChannelData>,
}

fn note_to_period(note: u8, fine: u8) -> u16 {
    FT_PERIOD_TABLE[fine as usize*36 + note as usize]
}

fn period_to_note(period: u16, fine: u8) -> u8 {
    for i in 0..36 {
        if period >= FT_PERIOD_TABLE[fine as usize + i] {
            return i as u8
        } 
    }
    35
}

impl FtPlayer {
    pub fn new(module: &Module, options: Options) -> Self {

        FtPlayer {
            options,

            ft_speed          : 6,
            ft_counter        : 0,
            ft_song_pos       : 0,
            ft_pbreak_pos     : 0,
            ft_pos_jump_flag  : false,
            ft_pbreak_flag    : false,
            ft_patt_del_time  : 0,
            ft_patt_del_time_2: 0,
            ft_pattern_pos    : 0,
            cia_tempo         : 125,

            ft_chantemp       : vec![ChannelData::new(); module.channels],
        }
    }

    fn ft_play_voice(&mut self, pat: usize, chn: usize, module: &ModData, mixer: &mut Mixer) {

        let event = module.patterns.event(pat, self.ft_pattern_pos, chn);
        let insnum = (((event.note & 0xf000) >> 8) | ((event.cmd as u16 & 0xf0) >> 4)) as u8;
        let note = event.note & 0xfff;
        let cmd = event.cmd & 0x0f;
        let cmdlo = event.cmdlo;
        
        {
            let ch = &mut self.ft_chantemp[chn];
            if cmd == 0 {
                ch.output_period = ch.n_period;
            } else {
                let ch_cmd = ch.n_command >> 4;
                if ch_cmd == 4 || ch_cmd == 6 {
                    if cmd != 4 && cmd != 6 {
                        ch.output_period = ch.n_period;
                    }
                }
            }
    
            ch.n_command = (cmd as u16) << 8 | cmdlo as u16;
    
            if insnum != 0 {
                let ins = insnum as usize - 1;
                ch.n_insnum = insnum;
                ch.n_volume = module.instruments[ins].volume;
                ch.output_volume = module.instruments[ins].volume;
                ch.n_finetune = module.instruments[ins].finetune;

	        mixer.set_sample(chn, ch.n_insnum as usize);
            }
    
            if cmd == 3 || cmd == 5 {   // check if tone portamento
                if note != 0 {
                    ch.n_wantedperiod = note;
                    if note == ch.n_period {
                        ch.n_toneportdirec = 0;
                    } else if note < ch.n_period {
                        ch.n_toneportdirec = 2;
                    } else {
                        ch.n_toneportdirec = 1;
                    }
                }
                if cmd != 5 {
                    if cmdlo != 0 {
                        ch.n_toneportspeed = cmdlo as u16;
                    }
                }
            } else {
                if note != 0 {
                    if cmd == 0x0e && cmdlo & 0xf0 == 0x50 {    // check if set finetune
                        ch.n_finetune = cmdlo & 0x0f;
                    }
                    ch.n_period = note;
                    if cmd == 0x0e && cmdlo & 0xf0 == 0xd0 {    // check if note delay
                        if cmdlo & 0x0f != 0 {
                            return
                        }
                    }
    
                    let ins = ch.n_insnum as usize - 1;
                    let mut length = module.instruments[ins].size;
                    if cmd == 9 {     // sample offset
                        let mut val = cmdlo;
                        if val == 0 {
                            val = ch.n_offset;
                        }
                        ch.n_offset = val;
    
			let l = (val as u16) << 8;
                        if l > length {
                            length = 0;
                        } else {
                            length -= l;
                        }
                    }
                    ch.n_length = length;
                    ch.n_loopstart = module.instruments[ins].repeat;
                    ch.n_replen = module.instruments[ins].replen;
                    ch.output_period = ch.n_period;
    
                    if ch.n_wavecontrol & 0x04 == 0 {
                        ch.n_vibratopos = 0;
                    }
                    if ch.n_wavecontrol & 0x40 == 0 {
                        ch.n_tremolopos = 0;
                    }
                }
            }
        }

        if cmd == 0 && cmdlo == 0 {
            return
        }

        match cmd {
            0x0c => self.ft_volume_change(chn, cmdlo),
            0x0e => self.ft_e_commands(chn, &module, cmdlo),
            0x0b => self.ft_position_jump(cmdlo),
            0x0d => self.ft_pattern_break(cmdlo),
            0x0f => self.ft_set_speed(cmdlo),
            _   => {},
        }
    }
        
    fn ft_e_commands(&mut self, chn: usize, module: &ModData, cmdlo: u8) {
        match cmdlo >> 4 {
            0x1 => self.ft_fine_porta_up(chn, cmdlo),
            0x2 => self.ft_fine_porta_down(chn, cmdlo),
            0x3 => self.ft_set_gliss_control(chn, cmdlo),
            0x4 => self.ft_set_vibrato_control(chn, cmdlo),
            0x7 => self.ft_set_tremolo_control(chn, cmdlo),
            0x9 => self.ft_retrig_note(chn, &module),
            0xa => self.ft_volume_fine_up(chn, cmdlo),
            0xb => self.ft_volume_fine_down(chn, cmdlo),
            0xc => self.ft_note_cut(chn, cmdlo),
            0x6 => self.ft_jump_loop(chn, cmdlo),
            0xe => self.ft_pattern_delay(chn, cmdlo),
            _   => {},
        }
    }

    fn ft_position_jump(&mut self, cmdlo: u8) {
        self.ft_song_pos = cmdlo.wrapping_sub(1);
        self.ft_pbreak_pos = 0;
        self.ft_pos_jump_flag = true;
    }

    fn ft_volume_change(&mut self, chn: usize, mut cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        if cmdlo > 64 {
            cmdlo = 64
        }
        ch.output_volume = cmdlo;
        ch.n_volume = cmdlo;
    }

    fn ft_pattern_break(&mut self, cmdlo: u8) {
        let row = (cmdlo >> 4) * 10 + (cmdlo & 0x0f);
        if row <= 63 {
            // mt_pj2
            self.ft_pbreak_pos = row;
        }
        self.ft_pos_jump_flag = true;
    }

    fn ft_set_speed(&mut self, cmdlo: u8) {
        if cmdlo < 0x20 {
            self.ft_speed = cmdlo;
            self.ft_counter = cmdlo;
        } else {
            self.cia_tempo = cmdlo;
        }
    }

    fn ft_fine_porta_up(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        let mut val = ch.n_period;
        val -= cmdlo as u16 & 0x0f;
        if val < 113 {
            val = 113
        }
        ch.n_period = val;
        ch.output_period = val;
    }

    fn ft_fine_porta_down(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        let mut val = ch.n_period;
        val += cmdlo as u16 & 0x0f;
        if val < 856 {
            val = 856
        }
        ch.n_period = val;
        ch.output_period = val;
    }

    fn ft_set_gliss_control(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        ch.n_gliss = cmdlo & 0x0f != 0;
    }

    fn ft_set_vibrato_control(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        ch.n_wavecontrol &= 0xf0;
        ch.n_wavecontrol |= cmdlo & 0x0f;
    }

    fn ft_jump_loop(&mut self, chn: usize, mut cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];

        cmdlo &= 0x0f;

        if cmdlo == 0 {
            ch.n_pattpos = self.ft_pattern_pos as u8;
        } else {
            if ch.n_loopcount == 0 {
                ch.n_loopcount = cmdlo;
                self.ft_pbreak_pos = ch.n_pattpos;
                self.ft_pbreak_flag = true;
            } else {
                ch.n_loopcount -= 1;
            }
        }
    }

    fn ft_set_tremolo_control(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        ch.n_wavecontrol &= 0x0f;
        ch.n_wavecontrol |= (cmdlo & 0x0f) << 4;
    }

    fn ft_retrig_note(&mut self, chn: usize, module: &ModData) {
        let ch = &mut self.ft_chantemp[chn];
        let ins = ch.n_insnum as usize - 1;

        ch.n_length = module.instruments[ins].size; 
        ch.n_loopstart = module.instruments[ins].repeat; 
        ch.n_replen = module.instruments[ins].replen; 
    }

    fn ft_volume_fine_up(&mut self, chn: usize, mut cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        cmdlo &= 0x0f;
        cmdlo += ch.n_volume;
        if cmdlo > 64 {
            cmdlo = 64;
        }
        ch.output_volume = cmdlo;
        ch.n_volume = cmdlo;
    }

    fn ft_volume_fine_down(&mut self, chn: usize, mut cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        cmdlo &= 0x0f;
        if cmdlo > ch.n_volume {
            cmdlo = 0
        } else {
            cmdlo = ch.n_volume - cmdlo;
        }
        ch.output_volume = cmdlo;
        ch.n_volume = cmdlo;
    }

    fn ft_note_cut(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        if cmdlo & 0x0f == 0 {
            ch.n_volume = 0;
            ch.output_volume = 0;
        }
    }

    fn ft_pattern_delay(&mut self, _chn: usize, cmdlo: u8) {
        if self.ft_patt_del_time_2 != 0 {
            return
        }
        self.ft_patt_del_time = (cmdlo & 0x0f) + 1;
    }

    fn ft_check_efx(&mut self, chn: usize, module: &ModData) {
        let cmd = ((self.ft_chantemp[chn].n_command & 0x0f00) >> 8) as u8;
        let cmdlo = (self.ft_chantemp[chn].n_command & 0xff) as u8;

        if cmd == 0 && cmdlo == 0 {
            return
        }

        match cmd {
            0xe => self.ft_more_e_commands(chn, &module, cmdlo),
            0x0 => self.ft_arpeggio(chn, cmdlo),
            0x1 => self.ft_porta_up(chn, cmdlo),
            0x2 => self.ft_porta_down(chn, cmdlo),
            0x3 => self.ft_tone_portamento(chn),
            0x4 => self.ft_vibrato(chn, cmdlo),
            0x5 => self.ft_tone_plus_vol_slide(chn, cmdlo),
            0x6 => self.ft_vibrato_plus_vol_slide(chn, cmdlo),
            0x7 => self.ft_tremolo(chn, cmdlo),
            0xa => self.ft_volume_slide(chn, cmdlo),
            _   => (),
        }
    }

    fn ft_volume_slide(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        let mut val = cmdlo >> 4;
        if val == 0 {
            val = cmdlo;
            if val > ch.n_volume {
                val = 0
            } else {
                val = ch.n_volume - val;
            }
        } else {
            val += ch.n_volume;
            if val > 64 {
                val = 64
            }
        }
        ch.output_volume = val;
        ch.n_volume = val;
    }

    fn ft_more_e_commands(&mut self, chn: usize, module: &ModData, cmdlo: u8) {
        match cmdlo >> 4 {
            0x09 => self.ft_retrig_note_2(chn, &module, cmdlo),
            0x0c => self.ft_note_cut_2(chn, cmdlo),
            0x0d => self.ft_note_delay_2(chn, &module, cmdlo),
            _   => {},
        }
    }

    fn ft_arpeggio(&mut self, chn: usize, cmdlo: u8) {
        if cmdlo == 0 {
            return
        }

        let val = match FT_ARPEGGIO_TABLE[self.ft_counter as usize] {
            0 => 0,
            1 => cmdlo >> 4,
            _ => cmdlo & 0x0f,
        };

        let period = {
            let ch = &mut self.ft_chantemp[chn];
            let note = period_to_note(ch.n_period, ch.n_finetune) + val;
            if note > 35 {
                ch.output_period = 0;
                return
            }

            note_to_period(note, ch.n_finetune)
        };

        self.ft_chantemp[chn].output_period = period;
    }

    fn ft_porta_up(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        let mut val = ch.n_period - cmdlo as u16;
        if val < 113 {
            val = 113
        }
        ch.n_period = val;
        ch.output_period = val;
    }

    fn ft_porta_down(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        let mut val = ch.n_period + cmdlo as u16;
        if val > 856 {
            val = 856
        }
        ch.n_period = val;
        ch.output_period = val;
    }

    fn ft_tone_portamento(&mut self, chn: usize) {
        let ch = &mut self.ft_chantemp[chn];

        let mut val = ch.n_period;

        if ch.n_toneportdirec > 1 {
            // porta up
            val -= ch.n_toneportspeed;
            if val < ch.n_wantedperiod {
                val = ch.n_wantedperiod;
                ch.n_toneportdirec = 1;
            }
        } else if ch.n_toneportdirec != 1 {
            return
        } else {
            // porta down
            val += ch.n_toneportspeed;
            if val > ch.n_wantedperiod {
                val = ch.n_wantedperiod;
                ch.n_toneportdirec = 1;
            }
        }

        ch.n_period = val;
        if ch.n_gliss {
            let note = period_to_note(val, ch.n_finetune);
            val = note_to_period(note, ch.n_finetune)
        }
        ch.output_period = val;
    }

    fn ft_vibrato(&mut self, chn: usize, cmdlo: u8) {
        if cmdlo != 0 {
            let ch = &mut self.ft_chantemp[chn];
            let depth = cmdlo & 0x0f;
            if depth != 0 {
                ch.n_vibratodepth = depth;
            }
            let speed = (cmdlo & 0xf0) >> 2;
            if speed != 0 {
                ch.n_vibratospeed = speed;
            }
        }

        self.ft_vibrato_2(chn)
    }

    fn ft_vibrato_2(&mut self, chn: usize) {
        let ch = &mut self.ft_chantemp[chn];
        let mut pos = (ch.n_vibratopos >> 2) & 0x1f;
        let val = match ch.n_wavecontrol & 0x03 {
            0 => {  // sine
                     FT_VIBRATO_TABLE[pos as usize]
                 },
            1 => {  // rampdown
                     pos <<= 3;
                     if ch.n_vibratopos & 0x80 != 0 { !pos } else { pos }
                 },
            _ => {  // square
                     255
                 }
        };
        let mut period = ch.n_period;
        let amt = (val as usize * ch.n_vibratodepth as usize) >> 7;
        if ch.n_vibratopos & 0x80 == 0 {
            period += amt as u16
        } else {
            period -= amt as u16
        };

        ch.output_period = period;
        ch.n_vibratopos = ch.n_vibratopos.wrapping_add(ch.n_vibratospeed);
    }

    fn ft_tone_plus_vol_slide(&mut self, chn: usize, cmdlo: u8) {
        self.ft_tone_portamento(chn);
        self.ft_volume_slide(chn, cmdlo);
    }

    fn ft_vibrato_plus_vol_slide(&mut self, chn: usize, cmdlo: u8) {
        self.ft_vibrato_2(chn);
        self.ft_volume_slide(chn, cmdlo);
    }

    fn ft_tremolo(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];
        if cmdlo != 0 {
            if cmdlo & 0x0f != 0 {
                 ch.n_tremolodepth = cmdlo & 0x0f
            }
            if cmdlo & 0xf0 != 0 {
                 ch.n_tremolospeed = cmdlo >> 4
            }
        }

        let mut pos = (ch.n_tremolopos >> 2) & 0x1f;
        let val = match (ch.n_wavecontrol >> 4) & 0x03 {
            0 => {  // sine
                     FT_VIBRATO_TABLE[pos as usize]
                 },
            1 => {  // rampdown
                     pos <<= 3;
                     if ch.n_vibratopos & 0x80 != 0 { !pos } else { pos }  // <-- bug in FT code
                 },
            _ => {  // square
                     255
                 },
        };

        let mut volume = ch.n_volume as isize;
        let amt = ((val as usize * ch.n_tremolodepth as usize) >> 6) as isize;
        if ch.n_tremolopos & 0x80 == 0 {
            volume += amt;
            if volume > 64 {
                volume = 64;
            }
        } else {
            volume -= amt;
            if volume < 0 {
                volume = 0;
            }
        }

        ch.output_volume = volume as u8;
        ch.n_tremolopos = ch.n_tremolopos.wrapping_add(ch.n_tremolospeed);
    }

    fn ft_retrig_note_2(&mut self, chn: usize, module: &ModData, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];

        if self.ft_speed - self.ft_counter == cmdlo & 0x0f {
            let ins = ch.n_insnum as usize - 1;
            ch.n_length = module.instruments[ins].size;
            ch.n_loopstart = module.instruments[ins].repeat;
            ch.n_replen = module.instruments[ins].replen;
            ch.output_period = ch.n_period;
        }
    }

    fn ft_note_cut_2(&mut self, chn: usize, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];

        if self.ft_speed - self.ft_counter == cmdlo & 0x0f {
            ch.output_volume = 0;
            ch.n_volume = 0;
        }
    }

    fn ft_note_delay_2(&mut self, chn: usize, module: &ModData, cmdlo: u8) {
        let ch = &mut self.ft_chantemp[chn];

        if self.ft_speed - self.ft_counter == cmdlo & 0x0f {
            let ins = ch.n_insnum as usize - 1;
            ch.n_length = module.instruments[ins].size;
            ch.n_loopstart = module.instruments[ins].repeat;
            ch.n_replen = module.instruments[ins].replen;
            ch.output_period = ch.n_period;
        }
    }

    fn ft_new_row(&mut self, module: &ModData, mixer: &mut Mixer) {

        if self.ft_counter != 1 {
            return
        }

        // mt_dskip
        self.ft_pattern_pos += 1;
        if self.ft_patt_del_time != 0 {
            self.ft_patt_del_time_2 = self.ft_patt_del_time;
            self.ft_patt_del_time = 0;
        }

        // mt_dskc
        if self.ft_patt_del_time_2 != 0 {
            self.ft_patt_del_time_2 -= 1;
            if self.ft_patt_del_time_2 != 0 {
                self.ft_pattern_pos -= 1;
            }
        }

        // mt_dska
        if self.ft_pbreak_flag {
            self.ft_pbreak_flag = false;
            self.ft_pattern_pos = self.ft_pbreak_pos;
            self.ft_pbreak_pos = 0;
        }

        // mt_nnpysk
        if self.ft_pattern_pos >= 64 {
            self.ft_next_position(&module);
        } else {
            self.ft_no_new_pos_yet(&module);
        }

        
    }

    fn ft_next_position(&mut self, module: &ModData) {
        self.ft_pattern_pos = self.ft_pbreak_pos;
        self.ft_pbreak_pos = 0;
        self.ft_pos_jump_flag = false;
        self.ft_song_pos = self.ft_song_pos.wrapping_add(1);
        self.ft_song_pos &= 0x7f;
        if self.ft_song_pos >= module.song_length {
            self.ft_song_pos = 0;
        }
    }

    fn ft_no_new_pos_yet(&mut self, module: &ModData) {
        if self.ft_pos_jump_flag {
            self.ft_next_position(&module);
            self.ft_no_new_pos_yet(&module);
        }
    }

    fn ft_music(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        self.ft_counter -= 1;
        if self.ft_counter != 0 {
            self.ft_no_new_all_channels(&module, &mut mixer);
            self.ft_new_row(&module, &mut mixer);
            return
        }

        self.ft_counter = self.ft_speed;
        self.ft_patt_del_time_2 = 0;
        if self.ft_patt_del_time_2 == 0 {
            self.ft_get_new_note(&module, &mut mixer)
        } else {
            self.ft_no_new_all_channels(&module, &mut mixer);
            self.ft_new_row(&module, &mut mixer);
        }
    }

    fn ft_no_new_all_channels(&mut self, module: &ModData, mixer: &mut Mixer) {
        for chn in 0..self.ft_chantemp.len() {
            self.ft_check_efx(chn, &module);
        }
    }

    fn ft_get_new_note(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        let pat = match module.pattern_in_position(self.ft_song_pos as usize) {
            Some(val) => val,
            None      => return,
        };

        for chn in 0..self.ft_chantemp.len() {
            self.ft_play_voice(pat, chn, &module, &mut mixer);
        }
    }

}

static FT_ARPEGGIO_TABLE: [u8; 32] = [
      0,   1,   2,   0,   1,   2,   0,   1,
      2,   0,   1,   2,   0,   1,   2,   0, 
      0,  24,  49,  74,  97, 120, 141, 161,     // buffer overflow values (vibrato table)
    180, 197, 212, 224, 235, 244, 250, 253
];

static FT_VIBRATO_TABLE: [u8; 32] = [
      0,  24,  49,  74,  97, 120, 141, 161,
    180, 197, 212, 224, 235, 244, 250, 253,
    255, 253, 250, 244, 235, 224, 212, 197,
    180, 161, 141, 120,  97,  74,  49,  24
];


static FT_PERIOD_TABLE: [u16; 16*36] = [
// Tuning 0, Normal
    856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
    428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
    214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113,
// Tuning 1
    850, 802, 757, 715, 674, 637, 601, 567, 535, 505, 477, 450,
    425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 239, 225,
    213, 201, 189, 179, 169, 159, 150, 142, 134, 126, 119, 113,
// Tuning 2
    844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474, 447,
    422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237, 224,
    211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118, 112,
// Tuning 3
    838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470, 444,
    419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235, 222,
    209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118, 111,
// Tuning 4
    832, 785, 741, 699, 660, 623, 588, 555, 524, 495, 467, 441,
    416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233, 220,
    208, 196, 185, 175, 165, 156, 147, 139, 131, 124, 117, 110,
// Tuning 5
    826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463, 437,
    413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232, 219,
    206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116, 109,
// Tuning 6
    820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460, 434,
    410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230, 217,
    205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115, 109,
// Tuning 7
    814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457, 431,
    407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228, 216,
    204, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114, 108,
// Tuning -8
    907, 856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480,
    453, 428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240,
    226, 214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120,
// Tuning -7
    900, 850, 802, 757, 715, 675, 636, 601, 567, 535, 505, 477,
    450, 425, 401, 379, 357, 337, 318, 300, 284, 268, 253, 238,
    225, 212, 200, 189, 179, 169, 159, 150, 142, 134, 126, 119,
// Tuning -6
    894, 844, 796, 752, 709, 670, 632, 597, 563, 532, 502, 474,
    447, 422, 398, 376, 355, 335, 316, 298, 282, 266, 251, 237,
    223, 211, 199, 188, 177, 167, 158, 149, 141, 133, 125, 118,
// Tuning -5
    887, 838, 791, 746, 704, 665, 628, 592, 559, 528, 498, 470,
    444, 419, 395, 373, 352, 332, 314, 296, 280, 264, 249, 235,
    222, 209, 198, 187, 176, 166, 157, 148, 140, 132, 125, 118,
// Tuning -4
    881, 832, 785, 741, 699, 660, 623, 588, 555, 524, 494, 467,
    441, 416, 392, 370, 350, 330, 312, 294, 278, 262, 247, 233,
    220, 208, 196, 185, 175, 165, 156, 147, 139, 131, 123, 117,
// Tuning -3
    875, 826, 779, 736, 694, 655, 619, 584, 551, 520, 491, 463,
    437, 413, 390, 368, 347, 328, 309, 292, 276, 260, 245, 232,
    219, 206, 195, 184, 174, 164, 155, 146, 138, 130, 123, 116,
// Tuning -2
    868, 820, 774, 730, 689, 651, 614, 580, 547, 516, 487, 460,
    434, 410, 387, 365, 345, 325, 307, 290, 274, 258, 244, 230,
    217, 205, 193, 183, 172, 163, 154, 145, 137, 129, 122, 115,
// Tuning -1
    862, 814, 768, 725, 684, 646, 610, 575, 543, 513, 484, 457,
    431, 407, 384, 363, 342, 323, 305, 288, 272, 256, 242, 228,
    216, 203, 192, 181, 171, 161, 152, 144, 136, 128, 121, 114
];


#[derive(Clone,Default)]
struct ChannelData {
    n_note         : u16,
    n_length       : u16,
    n_loopstart    : u16,
    n_replen       : u16,
    output_volume  : u8,
    n_finetune     : u8,
    output_period  : u16,
    n_insnum       : u8,
    n_wavecontrol  : u8,
    n_vibratopos   : u8,
    n_tremolopos   : u8,
    n_command      : u16,
    n_offset       : u8,
    n_period       : u16,
    n_wantedperiod : u16,
    n_toneportdirec: u8,
    n_gliss        : bool,
    n_toneportspeed: u16,
    n_vibratospeed : u8,
    n_vibratodepth : u8,
    n_pattpos      : u8,
    n_loopcount    : u8,
    n_tremolospeed : u8,
    n_tremolodepth : u8,
    n_volume       : u8,
}

impl ChannelData {
    pub fn new() -> Self {
        Default::default()
    }
}


impl FormatPlayer for FtPlayer {
    fn start(&mut self, data: &mut PlayerData, _mdata: &ModuleData, mixer: &mut Mixer) {

        //let module = mdata.as_any().downcast_ref::<ModData>().unwrap();

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
        mixer.set_pan(4, panl);
        mixer.set_pan(5, panr);
        mixer.set_pan(6, panr);
        mixer.set_pan(7, panl);

	self.ft_counter = 1;

    }

    fn play(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<ModData>().unwrap();

        self.ft_song_pos = data.pos as u8;
        self.ft_pattern_pos = data.row as u8;
        //self.ft_counter = self.ft_speed - data.frame as u8;

        self.ft_music(&module, &mut mixer);

	for chn in 0..self.ft_chantemp.len() {
            let ch = &mut self.ft_chantemp[chn];
	    mixer.set_loop_start(chn, ch.n_loopstart as u32 * 2);
	    mixer.set_loop_end(chn, (ch.n_loopstart + ch.n_replen) as u32 * 2);
	    mixer.enable_loop(chn, ch.n_replen > 1);
            mixer.set_period(chn, ch.output_period as f64);
            mixer.set_volume(chn, (ch.output_volume as usize) << 4);
        }

        data.frame = (self.ft_speed - self.ft_counter) as usize;
        data.row = self.ft_pattern_pos as usize;
        data.pos = self.ft_song_pos as usize;
        data.speed = self.ft_speed as usize;
        data.tempo = self.cia_tempo as usize;
    }

    fn reset(&mut self) {
        self.ft_speed           = 6;
        self.ft_counter         = 0;
        self.ft_song_pos        = 0;
        self.ft_pbreak_pos      = 0;
        self.ft_pos_jump_flag   = false;
        self.ft_pbreak_flag     = false;
        self.ft_patt_del_time   = 0;
        self.ft_patt_del_time_2 = 0;
        self.ft_pattern_pos     = 0;
    }
}
