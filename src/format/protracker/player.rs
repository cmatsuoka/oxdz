use module::Module;
use format::FormatPlayer;
use player::{PlayerData, Virtual};
use super::ModPatterns;

const FX_TONEPORTA: u8 = 0x03;

/// Vinterstigen PT2.1A Replayer
///
/// An oxdz player based on the Protracker V2.1A play routine written by Peter
/// "CRAYON" Hanning / Mushroom Studios in 1992. Original names are used whenever
/// possible (converted to snake case according to Rust convention, i.e.
/// mt_PosJumpFlag becomes mt_pos_jump_flag).

pub struct ModPlayer {
    name : &'static str,
    state: Vec<ChannelData>,

//  mt_speed          : u8,  // -> data.speed
//  mt_counter        : u8,  // -> data.frame
//  mt_song_pos       : u8,  // -> data.pos
    mt_pbreak_pos     : u8,
    mt_pos_jump_flag  : bool,
    mt_pbreak_flag    : bool,
    mt_low_mask       : u8,
    mt_patt_del_time  : u8,
    mt_patt_del_time_2: u8,
//  mt_pattern_pos    : u8,  // -> data.row
}

impl ModPlayer {
    pub fn new(module: &Module) -> Self {
        ModPlayer {
            name : r#""Vinterstigen" 0.1 PT2.1A replayer"#,
            state: vec![ChannelData::new(); module.chn],

//          mt_speed          : 0,
//          mt_counter        : 0,
//          mt_song_pos       : 0,
            mt_pbreak_pos     : 0,
            mt_pos_jump_flag  : false,
            mt_pbreak_flag    : false,
            mt_low_mask       : 0,
            mt_patt_del_time  : 0,
            mt_patt_del_time_2: 0,
        }
    }

    fn mt_music(&mut self, mut data: &mut PlayerData, module: &Module, mut virt: &mut Virtual) {
        let pats = module.patterns.as_any().downcast_ref::<ModPatterns>().unwrap();

        data.frame += 1;
        if data.frame >= data.speed {
            data.frame = 0;
            if self.mt_patt_del_time_2 == 0 {
                self.mt_get_new_note(&mut data, &module, &pats, &mut virt);
            } else {
                self.mt_no_new_all_channels(&mut data, &pats, &mut virt);

                // mt_dskip
                data.pos +=1;
                if self.mt_patt_del_time != 0 {
                    self.mt_patt_del_time_2 = self.mt_patt_del_time;
                    self.mt_patt_del_time = 0;
                }

                // mt_dskc
                if self.mt_patt_del_time_2 != 0 {
                    self.mt_patt_del_time_2 -= 1;
                    if self.mt_patt_del_time_2 != 0 {
                        data.row -= 1;
                    }
                }

                // mt_dska
                if self.mt_pbreak_flag {
                    self.mt_pbreak_flag = false;
                    data.row = self.mt_pbreak_pos as usize;
                    self.mt_pbreak_pos = 0;
                }

                // mt_nnpysk
                if data.row >= 64 {
                    self.mt_next_position(&mut data, &module);
                }
                self.mt_no_new_pos_yet(&mut data, &module);
            }
        } else {
            // mt_NoNewNote
            self.mt_no_new_all_channels(&mut data, &pats, &mut virt);
            self.mt_no_new_pos_yet(&mut data, &module);
            return;
        }
    }

    fn mt_no_new_all_channels(&mut self, mut data: &mut PlayerData, pats: &ModPatterns, mut virt: &mut Virtual) {
        for chn in 0..self.state.len() {
            let event = pats.event(data.pos, data.row, chn);
            self.mt_check_efx(chn, &mut data, event.cmdlo, &mut virt);
        }
    }

    fn mt_get_new_note(&mut self, mut data: &mut PlayerData, module: &Module, pats: &ModPatterns, mut virt: &mut Virtual) {
        for chn in 0..self.state.len() {
            // mt_PlayVoice
            let event = pats.event(data.pos, data.row, chn);
            if event.has_ins() {
                let instrument = &module.instrument[event.ins as usize];
                virt.set_patch(chn, event.ins as usize, event.ins as usize, event.note as usize);
                virt.set_volume(chn, instrument.volume);
            }

            // mt_SetRegs
            if event.has_note() {

                let period = self.state[chn].n_period as f64;

                match event.cmd {
                    0xe => if (event.cmdlo & 0xf0) == 0x50 {
                                // mt_DoSetFinetune()
                           },
                    0x3 => {
                               self.mt_set_tone_porta(chn, &mut data);
                               self.mt_check_efx(chn, &mut data, event.cmdlo, &mut virt)
                           },
                    0x5 => {
                               self.mt_set_tone_porta(chn, &mut data);
                               self.mt_check_efx(chn, &mut data, event.cmdlo, &mut virt)
                           },
                    0x9 => {
                               self.mt_check_more_efx(chn, &mut data, event.cmdlo, &mut virt);
                               virt.set_period(chn, period)
                           },
                    _   => virt.set_period(chn, period),
                }
                

            } else {
                self.mt_check_more_efx(chn, &mut data, event.cmdlo, &mut virt);
            }
        }
    }

    fn mt_next_position(&mut self, mut data: &mut PlayerData, module: &Module) {
        data.row = self.mt_pbreak_pos as usize;
        self.mt_pbreak_pos = 0;
        self.mt_pos_jump_flag = false;
        data.pos += 1;
        data.pos &= 0x7f;
        if data.pos >= module.len(0) {
            data.pos = 0;
        }
    }

    fn mt_no_new_pos_yet(&mut self, mut data: &mut PlayerData, module: &Module) {
        if self.mt_pos_jump_flag {
            self.mt_next_position(&mut data, &module);
            self.mt_no_new_pos_yet(&mut data, &module);
        }
    }

    fn mt_check_efx(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8, mut virt: &mut Virtual) {
        let cmd = 0;

        // mt_UpdateFunk()
        if cmd == 000 {
            self.per_nop(chn, &mut data);
            return
        }

        match cmd {
            0x0 => self.mt_arpeggio(chn, &mut data),
            0x1 => self.mt_porta_up(chn, &mut data, cmdlo, &mut virt),
            0x2 => self.mt_porta_down(chn, &mut data, cmdlo, &mut virt),
            0x3 => self.mt_tone_portamento(chn, &mut data),
            0x4 => self.mt_vibrato(chn, &mut data),
            0x5 => self.mt_tone_plus_vol_slide(chn, &mut data),
            0x6 => self.mt_vibrato_plus_vol_slide(chn, &mut data),
            0xe => self.mt_e_commands(chn, &mut data, cmdlo, &mut virt),
// SetBack MOVE.W  n_period(A6),6(A5)
            0x7 => self.mt_tremolo(chn, &mut data),
            0xa => self.mt_volume_slide(chn, &mut data),
            _   => {},
        }
    }

    fn per_nop(&self, chn: usize, mut data: &mut PlayerData) {
        //self.state.n_period = period
    }

    fn mt_arpeggio(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_fine_porta_up(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8, mut virt: &mut Virtual) {
        if data.frame != 0 {
            return
        }
        self.mt_low_mask = 0x0f;
        self.mt_porta_up(chn, &mut data, cmdlo, &mut virt);
    }

    fn mt_porta_up(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8, mut virt: &mut Virtual) {
        let mut period = self.state[chn].n_period;
        period -= (cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if period < 113 {
            period = 113;
        }
        self.state[chn].n_period = period;
        virt.set_period(chn, period as f64);  // MOVE.W  D0,6(A5)
    }

    fn mt_fine_porta_down(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8, mut virt: &mut Virtual) {
        if data.frame != 0 {
            return
        }
        self.mt_low_mask = 0x0f;
        self.mt_porta_down(chn, &mut data, cmdlo, &mut virt);
    }

    fn mt_porta_down(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8, mut virt: &mut Virtual) {
        let mut period = self.state[chn].n_period;
        period += (cmdlo & self.mt_low_mask) as u16;
        self.mt_low_mask = 0xff;
        if period < 856 {
            period = 856;
        }
        self.state[chn].n_period = period;
        virt.set_period(chn, period as f64);  // MOVE.W  D0,6(A5)
    }

    fn mt_set_tone_porta(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_clear_tone_porta(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_tone_portamento(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_vibrato(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_tone_plus_vol_slide(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_vibrato_plus_vol_slide(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_tremolo(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_sample_offset(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_volume_slide(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_position_jump(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_volume_change(&mut self, chn: usize, mut data: &mut PlayerData, mut cmdlo: u8, virt: &mut Virtual) {
        if cmdlo > 0x40 {
            cmdlo = 40
        }
        self.state[chn].n_volume = cmdlo;
        virt.set_volume(chn, cmdlo as usize);  // MOVE.W  D0,8(A5)
    }

    fn mt_pattern_break(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8) {
        let line = (cmdlo >> 4) * 10 + (cmdlo & 0x0f);
        if line >= 63 {
            // mt_pj2
            self.mt_pbreak_pos = 0;
        }
        self.mt_pos_jump_flag = true;
    }

    fn mt_set_speed(&self, chn: usize, mut data: &mut PlayerData, cmdlo: u8) {
        if cmdlo != 0 {
            data.frame = 0;
            data.speed = cmdlo as usize;
        }
    }

    fn mt_check_more_efx(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8, mut virt: &mut Virtual) {
        let cmd = 0;

        // mt_UpdateFunk()
        match cmd {
            0x9 => self.mt_sample_offset(chn, &mut data),
            0xb => self.mt_position_jump(chn, &mut data),
            0xd => self.mt_pattern_break(chn, &mut data, cmdlo),
            0xe => self.mt_e_commands(chn, &mut data, cmdlo, &mut virt),
            0xf => self.mt_set_speed(chn, &mut data, cmdlo),
            0xc => self.mt_volume_change(chn, &mut data, cmdlo, &mut virt),
            _   => {},
        }

        // per_nop
        self.per_nop(chn, &mut data)
    }

    fn mt_e_commands(&mut self, chn: usize, mut data: &mut PlayerData, cmdlo: u8, mut virt: &mut Virtual) {
        let cmd = 0;

        match cmd {
           0x0 => self.mt_filter_on_off(chn, &mut data),
           0x1 => self.mt_fine_porta_up(chn, &mut data, cmdlo, &mut virt),
           0x2 => self.mt_fine_porta_down(chn, &mut data, cmd, &mut virt),
           0x3 => self.mt_set_gliss_control(chn, &mut data),
           0x4 => self.mt_set_vibrato_control(chn, &mut data),
           0x5 => self.mt_set_finetune(chn, &mut data),
           0x6 => self.mt_jump_loop(chn, &mut data),
           0x7 => self.mt_set_tremolo_control(chn, &mut data),
           0x9 => self.mt_retrig_note(chn, &mut data),
           0xa => self.mt_volume_fine_up(chn, &mut data),
           0xb => self.mt_volume_fine_down(chn, &mut data),
           0xc => self.mt_note_cut(chn, &mut data),
           0xd => self.mt_note_delay(chn, &mut data),
           0xe => self.mt_pattern_delay(chn, &mut data),
           0xf => self.mt_funk_it(chn, &mut data),
           _   => {},
        }
    }

    fn mt_filter_on_off(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_set_gliss_control(&self, chn: usize, mut data: &mut PlayerData) {
        //state.n_glissfunk = parm;
    }

    fn mt_set_vibrato_control(&self, chn: usize, mut data: &mut PlayerData) {
        //state.n_wavecontrol &= 0xf0;
        //state.n_wavecontrol |= parm & 0x0f;
    }

    fn mt_set_finetune(&self, chn: usize, mut data: &mut PlayerData) {
        //state.n_finetune = parm;
    }

    fn mt_jump_loop(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_set_tremolo_control(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_retrig_note(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_volume_fine_up(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_volume_fine_down(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_note_cut(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_note_delay(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_pattern_delay(&self, chn: usize, mut data: &mut PlayerData) {
    }

    fn mt_funk_it(&self, chn: usize, mut data: &mut PlayerData) {
    }
}

impl FormatPlayer for ModPlayer {
    fn name(&self) -> &'static str {
        self.name
    }

    fn play(&mut self, mut data: &mut PlayerData, module: &Module, mut virt: &mut Virtual) {
        self.mt_music(&mut data, &module, &mut virt)
    }

    fn reset(&mut self) {
        self.mt_pbreak_pos      = 0;
        self.mt_pos_jump_flag   = false;
        self.mt_pbreak_flag     = false;
        self.mt_low_mask        = 0;
        self.mt_patt_del_time   = 0;
        self.mt_patt_del_time_2 = 0;
    }
}


#[derive(Clone,Default)]
struct ChannelData {
    n_note         : u8,
    n_cmd          : u8,
    n_cmdlo        : u8,
    n_period       : u16,
    n_finetune     : i8,
    n_volume       : u8,
    n_toneportdirec: i8,
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
    n_reallength   : u16,
}

impl ChannelData {
    pub fn new() -> Self {
        Default::default()
    }
}
