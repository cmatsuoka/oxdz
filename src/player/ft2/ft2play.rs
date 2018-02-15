use std::cmp;
use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer};
use format::xm::XmData;
use mixer::Mixer;

const IS_VOL     : u8 = 1;
const IS_PERIOD  : u8 = 2;
const IS_NYTON   : u8 = 4;
const IS_PAN     : u8 = 8;
const IS_QUICKVOL: u8 = 16;

#[derive(Default)]
struct SongTyp {
    len            : u16,
    rep_s          : u16,
    ant_chn        : u8,
    ant_ptn        : u16,
    ant_instrs     : u16,
    song_pos       : i16,
    patt_nr        : i16,
    patt_pos       : i16,
    patt_len       : i16,
    speed          : u16,
    tempo          : u16,
    glob_vol       : u16,
    timer          : u16,
    patt_del_time  : u8,
    patt_del_time_2: u8,
    p_break_flag   : bool,
    p_break_pos    : u8,
    pos_jump_flag  : bool,
    //song_tab       : [u8; 256],
    ver            : u16,
    name           : String,
}

#[derive(Default)]
struct SampleTyp {
    len      : i32,
    rep_s    : i32,
    rep_l    : i32,
    vol      : u8,
    fine     : i8,
    typ      : u8,
    pan      : u8,
    rel_ton  : i8,
    skrap    : u8,
    name     : String,
    //pek      : i8,
    //gus_base : i32,
    //gus_len  : i32,
    fixed    : u8,
    fix_spar : u16,
    //res1     : u8,
    fixed_pos: i32,
}

struct InstrTyp {
    ta         : [u8; 96],
    env_vp     : [[i16; 2]; 12],
    env_pp     : [[i16; 2]; 12],
    env_vp_ant : u8,
    env_pp_ant : u8,
    env_v_sust : u8,
    env_v_rep_s: u8,
    env_v_rep_e: u8,
    env_p_sust : u8,
    env_p_rep_s: u8,
    env_p_rep_e: u8,
    env_v_typ  : u8,
    env_p_typ  : u8,
    vib_typ    : u8,
    vib_sweep  : u8,
    vib_depth  : u8,
    vib_rate   : u8,
    fade_out   : u16,
    //midi_on     : bool,
    //midi_channel: u8,
    //midi_program: i16,
    //midi_bend   : i16,
    mute       : bool,
    samp       : [SampleTyp; 32],
}

#[derive(Default)]
struct StmTyp {
    out_vol               : i8,
    real_vol              : i8,
    rel_ton_nr            : i8,
    fine_tune             : i8,
    old_vol               : i8, //u8,
    old_pan               : u8,
    out_pan               : u8,
    final_pan             : u8,
    env_sustain_active    : bool,
    eff_typ               : u8,
    eff                   : u8,
    smp_offset            : u8,
    wave_ctrl             : u8,
    status                : u8,
    porta_dir             : u8,
    gliss_funk            : u8,
    vib_pos               : u8,
    trem_pos              : u8,
    vib_speed             : u8,
    vib_depth             : u8,
    trem_speed            : u8,
    trem_depth            : u8,
    patt_pos              : u8,
    loop_cnt              : u8,
    vol_slide_speed       : u8,
    f_vol_slide_up_speed  : u8,
    f_vol_slide_down_speed: u8,
    f_porta_up_speed      : u8,
    f_porta_down_speed    : u8,
    e_porta_up_speed      : u8,
    e_porta_down_speed    : u8,
    porta_up_speed        : u8,
    porta_down_speed      : u8,
    retrig_speed          : u8,
    retrig_cnt            : u8,
    retrig_vol            : u8,
    vol_kol_vol           : u8,
    ton_nr                : u8,
    env_v_pos             : u8,
    e_vib_pos             : u8,
    env_p_pos             : u8,
    tremor_save           : u8,
    tremor_pos            : u8,
    glob_vol_slide_speed  : u8,
    panning_slide_speed   : u8,
    mute                  : bool,
    real_period           : i16,
    env_v_ip_value        : i16,
    env_p_ip_value        : i16,
    smp_start_pos         : u16,
    instr_nr              : u16,
    ton_typ               : u16,
    final_vol             : u16,
    fade_out_speed        : u16,
    env_v_cnt             : u16,
    env_v_amp             : u16,
    env_p_cnt             : u16,
    env_p_amp             : u16,
    e_vib_amp             : u16,
    e_vib_sweep           : u16,
    porta_speed           : u16,
    want_period           : u16,
    final_period          : u16,
    out_period            : u16,
    fade_out_amp          : u32,

    smp_ptr               : usize,  // oxdz: store these as indexes instead of ponters
    instr_ptr             : usize,
}

struct TonTyp {
    ton    : u8,
    instr  : u8,
    vol    : u8,
    eff_typ: u8,
    eff    : u8,
}

const MAX_NOTES : usize = (12 * 10 * 16) + 16;
const MAX_VOICES: usize = 32;


// TABLES AND VARIABLES
static PANNING_TAB: [u32; 257] = [
        0,  4096,  5793,  7094,  8192,  9159, 10033, 10837, 11585, 12288, 12953, 13585, 14189, 14768, 15326, 15864,
    16384, 16888, 17378, 17854, 18318, 18770, 19212, 19644, 20066, 20480, 20886, 21283, 21674, 22058, 22435, 22806,
    23170, 23530, 23884, 24232, 24576, 24915, 25249, 25580, 25905, 26227, 26545, 26859, 27170, 27477, 27780, 28081,
    28378, 28672, 28963, 29251, 29537, 29819, 30099, 30377, 30652, 30924, 31194, 31462, 31727, 31991, 32252, 32511,
    32768, 33023, 33276, 33527, 33776, 34024, 34270, 34514, 34756, 34996, 35235, 35472, 35708, 35942, 36175, 36406,
    36636, 36864, 37091, 37316, 37540, 37763, 37985, 38205, 38424, 38642, 38858, 39073, 39287, 39500, 39712, 39923,
    40132, 40341, 40548, 40755, 40960, 41164, 41368, 41570, 41771, 41972, 42171, 42369, 42567, 42763, 42959, 43154,
    43348, 43541, 43733, 43925, 44115, 44305, 44494, 44682, 44869, 45056, 45242, 45427, 45611, 45795, 45977, 46160,
    46341, 46522, 46702, 46881, 47059, 47237, 47415, 47591, 47767, 47942, 48117, 48291, 48465, 48637, 48809, 48981,
    49152, 49322, 49492, 49661, 49830, 49998, 50166, 50332, 50499, 50665, 50830, 50995, 51159, 51323, 51486, 51649,
    51811, 51972, 52134, 52294, 52454, 52614, 52773, 52932, 53090, 53248, 53405, 53562, 53719, 53874, 54030, 54185,
    54340, 54494, 54647, 54801, 54954, 55106, 55258, 55410, 55561, 55712, 55862, 56012, 56162, 56311, 56459, 56608,
    56756, 56903, 57051, 57198, 57344, 57490, 57636, 57781, 57926, 58071, 58215, 58359, 58503, 58646, 58789, 58931,
    59073, 59215, 59357, 59498, 59639, 59779, 59919, 60059, 60199, 60338, 60477, 60615, 60753, 60891, 61029, 61166,
    61303, 61440, 61576, 61712, 61848, 61984, 62119, 62254, 62388, 62523, 62657, 62790, 62924, 63057, 63190, 63323,
    63455, 63587, 63719, 63850, 63982, 64113, 64243, 64374, 64504, 64634, 64763, 64893, 65022, 65151, 65279, 65408,
    65536
];

static AMIGA_FINE_PERIOD: [u16; 12 * 8] = [
    907, 900, 894, 887, 881, 875, 868, 862, 856, 850, 844, 838,
    832, 826, 820, 814, 808, 802, 796, 791, 785, 779, 774, 768,
    762, 757, 752, 746, 741, 736, 730, 725, 720, 715, 709, 704,
    699, 694, 689, 684, 678, 675, 670, 665, 660, 655, 651, 646,
    640, 636, 632, 628, 623, 619, 614, 610, 604, 601, 597, 592,
    588, 584, 580, 575, 570, 567, 563, 559, 555, 551, 547, 543,
    538, 535, 532, 528, 524, 520, 516, 513, 508, 505, 502, 498,
    494, 491, 487, 484, 480, 477, 474, 470, 467, 463, 460, 457
];

static VIB_TAB: [u8; 32] = [
    0,   24,   49,  74,  97, 120, 141, 161,
    180, 197, 212, 224, 235, 244, 250, 253,
    255, 253, 250, 244, 235, 224, 212, 197,
    180, 161, 141, 120,  97,  74,  49,  24
];




#[derive(Default)]
pub struct Ft2Play<'a> {
    linear_frq_tab      : bool,
    speed_val           : u32,
    real_replay_rate    : u32,
    f_audio_freq        : f32,
    quick_vol_ramp_mul_f: f32,
    tick_vol_ramp_mul_f : f32,
    song                : SongTyp,

    stm                 : [StmTyp; MAX_VOICES],
    instr               : Vec<&'a InstrTyp>,

    note2period         : Vec<i16>,
    log_tab             : Vec<u32>,
}

impl<'a> Ft2Play<'a> {
    pub fn new(_module: &Module, _options: Options) -> Self {
        Default::default()
    }

    // CODE START
    fn set_speed(&mut self, bpm: u16) {
        if bpm > 0 {
            self.speed_val = ((self.real_replay_rate * 5) / 2) / bpm as u32;
            self.tick_vol_ramp_mul_f = 1.0 / self.speed_val as f32;
        }
    }

    fn retrig_volume(&mut self, chn: usize) {
        let ch = &mut self.stm[chn];
        ch.real_vol = ch.old_vol;
        ch.out_vol  = ch.old_vol;
        ch.out_pan  = ch.old_pan;
        ch.status  |= IS_VOL + IS_PAN + IS_QUICKVOL;
    }

    fn retrig_envelope_vibrato(&mut self, chn: usize) {
        let ch = &mut self.stm[chn];

        if ch.wave_ctrl & 0x04 == 0 {
		ch.vib_pos  = 0;
	}
        if ch.wave_ctrl & 0x40 == 0 {
		ch.trem_pos = 0;
	}

        ch.retrig_cnt = 0;
        ch.tremor_pos = 0;

        ch.env_sustain_active = true;

        let ins = &self.instr[ch.instr_ptr];

        if ins.env_v_typ & 1 != 0 {
            ch.env_v_cnt = 65535;
            ch.env_v_pos = 0;
        }

        if ins.env_p_typ & 1 != 0 {
            ch.env_p_cnt = 65535;
            ch.env_p_pos = 0;
        }

        ch.fade_out_speed = ins.fade_out;  // FT2 doesn't check if fadeout is more than 4095
        ch.fade_out_amp   = 32768;

        if ins.vib_depth != 0 {
            ch.e_vib_pos = 0;

            if ins.vib_sweep != 0 {
                ch.e_vib_amp   = 0;
                ch.e_vib_sweep = ((ins.vib_depth as u16) << 8) / ins.vib_sweep as u16;
            } else {
                ch.e_vib_amp   = (ins.vib_depth as u16) << 8;
                ch.e_vib_sweep = 0;
            }
        }
    }

    fn key_off(&mut self, chn: usize) {
        let ch = &mut self.stm[chn];

        ch.env_sustain_active = false;

        let ins = &self.instr[ch.instr_ptr];

        if ins.env_p_typ & 1 == 0 {  // yes, FT2 does this (!)
            if ch.env_p_cnt >= ins.env_pp[ch.env_p_pos as usize][0] as u16 {
                ch.env_p_cnt = ins.env_pp[ch.env_p_pos as usize][0] as u16 - 1;
            }
        }

        if ins.env_v_typ & 1 != 0 {
            if ch.env_v_cnt >= ins.env_vp[ch.env_v_pos as usize][0] as u16 {
                ch.env_v_cnt = ins.env_vp[ch.env_v_pos as usize][0] as u16 - 1;
            }
        } else {
            ch.real_vol = 0;
            ch.out_vol  = 0;
            ch.status |= IS_VOL + IS_QUICKVOL;
        }
    }

    fn get_frequence_value(&self, period: u16) -> u32 {
        if period == 0 {
            return 0
        }

        let mut rate: u32;

        if self.linear_frq_tab {
            let index = (12 * 192 * 4) - period;
            rate = self.log_tab[index as usize % (12 * 16 * 4)];

            let shift = (14 - (index / (12 * 16 * 4))) & 0x1F;
            if shift > 0 {
                rate >>= shift;
            }
        } else {
            rate = (1712 * 8363) / period as u32;
        }

        return rate;
    }

    fn start_tone(&mut self, mut ton: u8, eff_typ: u8, eff: u8, chn: usize) {

        // no idea if this EVER triggers...
        if ton == 97 {
            self.key_off(chn);
            return
        }
        // ------------------------------------------------------------

        let ch = &mut self.stm[chn];

        // if we came from Rxy (retrig), we didn't check note (Ton) yet
        if ton == 0 {
            ton = ch.ton_nr;
            if ton == 0 {
                return  // if still no note, return.
            }
        }
        // ------------------------------------------------------------

        ch.ton_nr = ton;

        let ins = self.instr[ch.instr_nr as usize];

        //ch.instr_ptr = ins;
        ch.instr_ptr = ch.instr_nr as usize;

        ch.mute = ins.mute;

        if ton > 95 {  // added security check
            ton = 95;
        }

        let smp = (ins.ta[ton as usize - 1] & 0x0F) as usize;

        let s = &ins.samp[smp];
        ch.smp_ptr = smp;

        ch.rel_ton_nr = s.rel_ton;

        ton = (ton as i16 + ch.rel_ton_nr as i16) as u8;
        if ton >= (12 * 10) {
            return
        }

        ch.old_vol = s.vol as i8;
        ch.old_pan = s.pan;

        ch.fine_tune = if eff_typ == 0x0E && (eff & 0xF0) == 0x50 {
            ((eff & 0x0F) * 16) as i8 - 128  // result is now -128 .. 127
        } else {
            s.fine
        };

        if ton > 0 {
            // MUST use >> 3 here (sar cl,3) - safe for x86/x86_64
            let tmp_ton = ((ton - 1) * 16) + ((((ch.fine_tune >> 3) + 16) as u8 )& 0xFF);

            // oxdz: can't happen, tmp_ton is limited by type size
            /*
            if tmp_ton < MAX_NOTES {  // should never happen, but FT2 does this check
                ch.real_period = self.note2period[tmp_ton as usize];
                ch.out_period  = ch.real_period as u16;
            }
            */
        }

        ch.status |= IS_PERIOD + IS_VOL + IS_PAN + IS_NYTON + IS_QUICKVOL;

        if eff_typ == 9 {
            if eff != 0 {
                ch.smp_offset = ch.eff;
            }

            ch.smp_start_pos = 256 * ch.smp_offset as u16;
        } else {
            ch.smp_start_pos = 0;
        }
    }

    fn multi_retrig(&mut self, chn: usize) {
        {
            let ch = &mut self.stm[chn];

            let cnt = ch.retrig_cnt + 1;
            if cnt < ch.retrig_speed {
                ch.retrig_cnt = cnt;
                return
            }

            ch.retrig_cnt = 0;

            let mut vol = ch.real_vol;
            let cmd = ch.retrig_vol;

            // 0x00 and 0x08 are not handled, ignore them

            vol = match cmd {
                0x01 => cmp::max(vol - 1, 0),
                0x02 => cmp::max(vol - 2, 0),
                0x03 => cmp::max(vol - 4, 0),
                0x04 => cmp::max(vol - 8, 0),
                0x05 => cmp::max(vol - 16, 0),
                0x06 => vol>>1 + vol>>3 + vol>>4,
                0x07 => vol >> 1,
                0x09 => cmp::min(vol + 1, 64),
                0x0a => cmp::min(vol + 2, 64),
                0x0b => cmp::min(vol + 4, 64),
                0x0c => cmp::min(vol + 8, 64),
                0x0d => cmp::min(vol + 16, 64),
                0x0e => cmp::min(vol>>1 + vol, 64),
                0x0f => cmp::min(vol + vol, 64),
                _    => vol,
            };

            ch.real_vol = vol;
            ch.out_vol  = ch.real_vol;

            if ch.vol_kol_vol >= 0x10 && ch.vol_kol_vol <= 0x50 {
                ch.out_vol  = ch.vol_kol_vol as i8 - 0x10;
                ch.real_vol = ch.out_vol;
            } else if ch.vol_kol_vol >= 0xC0 && ch.vol_kol_vol <= 0xCF {
                ch.out_pan = (ch.vol_kol_vol & 0x0F) << 4;
            }
        }

        self.start_tone(0, 0, 0, chn);
    }

    fn check_more_effects(&mut self, chn: usize) {
        let mut set_speed = false;
        let mut set_global_volume = false;
        {
            let ch = &mut self.stm[chn];
            let ins = &self.instr[ch.instr_ptr];

            // Bxx - position jump
            if ch.eff_typ == 11 {
                self.song.song_pos      = ch.eff as i16 - 1;
                self.song.p_break_pos   = 0;
                self.song.pos_jump_flag = true;
            }

            // Dxx - pattern break
            else if ch.eff_typ == 13 {
                self.song.pos_jump_flag = true;

                let tmp_eff = (ch.eff>>4)*10 + ch.eff&0x0F;
                self.song.p_break_pos = if tmp_eff <= 63 {
                    tmp_eff
                } else {
                    0
                }
            }

            // Exx - E effects
            else if ch.eff_typ == 14 {
                // E1x - fine period slide up
                if ch.eff & 0xF0 == 0x10 {
                    let mut tmp_eff = ch.eff & 0x0F;
                    if tmp_eff == 0 {
                        tmp_eff = ch.f_porta_up_speed;
                    }

                    ch.f_porta_up_speed = tmp_eff;

                    ch.real_period -= tmp_eff as i16 * 4;
                    if ch.real_period < 1 {
                        ch.real_period = 1
                    }

                    ch.out_period = ch.real_period as u16;
                    ch.status    |= IS_PERIOD;
                }

                // E2x - fine period slide down
                else if ch.eff & 0xF0 == 0x20 {
                    let mut tmp_eff = ch.eff & 0x0F;
                    if tmp_eff == 0 {
                        tmp_eff = ch.f_porta_down_speed;
                    }

                    ch.f_porta_down_speed = tmp_eff;

                    ch.real_period += tmp_eff as i16 * 4;
                    if ch.real_period > 32000 - 1 {
                        ch.real_period = 32000 - 1;
                    }

                    ch.out_period = ch.real_period as u16;
                    ch.status   |= IS_PERIOD;
                }

                // E3x - set glissando type
                else if ch.eff & 0xF0 == 0x30 {
                    ch.gliss_funk = ch.eff & 0x0F;
                }

                // E4x - set vibrato waveform
                else if ch.eff & 0xF0 == 0x40 {
                    ch.wave_ctrl = (ch.wave_ctrl & 0xF0) | (ch.eff & 0x0F);
                }

                // E5x (set finetune) is handled in StartTone()

                // E6x - pattern loop
                else if ch.eff & 0xF0 == 0x60 {
                    if ch.eff == 0x60 {  // E60, empty param
                        ch.patt_pos = (self.song.patt_pos & 0x00FF) as u8;
                    } else {
                        if ch.loop_cnt == 0 {
                            ch.loop_cnt = ch.eff & 0x0F;

                            self.song.p_break_pos  = ch.patt_pos;
                            self.song.p_break_flag = true;
                        } else {
                            ch.loop_cnt -= 1;
                            if ch.loop_cnt != 0 {
                                self.song.p_break_pos  = ch.patt_pos;
                                self.song.p_break_flag = true;
                            }
                        }
                    }
                }

                // E7x - set tremolo waveform
                else if ch.eff & 0xF0 == 0x70 {
                    ch.wave_ctrl = (ch.eff & 0x0F) << 4 | (ch.wave_ctrl & 0x0F);
                }

                // E8x - set 4-bit panning (NON-FT2)
                else if ch.eff & 0xF0 == 0x80 {
                    ch.out_pan = (ch.eff & 0x0F) * 16;
                    ch.status |= IS_PAN;
                }

                // EAx - fine volume slide up
                else if ch.eff & 0xF0 == 0xA0 {
                    let mut tmp_eff = ch.eff & 0x0F;
                    if tmp_eff == 0 {
                        tmp_eff = ch.f_vol_slide_up_speed;
                    }

                    ch.f_vol_slide_up_speed = tmp_eff;

                    // unsigned clamp
                    if ch.real_vol <= 64 - tmp_eff as i8 {
                        ch.real_vol += tmp_eff as i8;
                    } else {
                        ch.real_vol = 64;
                    }

                    ch.out_vol = ch.real_vol;
                    ch.status |= IS_VOL;
                }

                // EBx - fine volume slide down
                else if (ch.eff & 0xF0) == 0xB0 {
                    let mut tmp_eff = ch.eff & 0x0F;
                    if tmp_eff == 0 {
                        tmp_eff = ch.f_vol_slide_down_speed;
                    }

                    ch.f_vol_slide_down_speed = tmp_eff;

                    // unsigned clamp
                    if ch.real_vol >= tmp_eff as i8 {
                        ch.real_vol -= tmp_eff as i8
                    } else {
                        ch.real_vol = 0
                    }

                    ch.out_vol = ch.real_vol;
                    ch.status |= IS_VOL;
                }

                // ECx - note cut
                else if ch.eff & 0xF0 == 0xC0 {
                    if ch.eff == 0xC0 {  // empty param
                        ch.real_vol = 0;
                        ch.out_vol = 0;
                        ch.status |= IS_VOL + IS_QUICKVOL;
                    }
                }

                // EEx - pattern delay
                else if ch.eff & 0xF0 == 0xE0 {
                    if self.song.patt_del_time_2 == 0 {
                        self.song.patt_del_time = ch.eff & 0x0F + 1;
                    }
                }
            }

            // Fxx - set speed/tempo
            else if ch.eff_typ == 15 {
                if ch.eff >= 32 {
                    self.song.speed = ch.eff as u16;
                    set_speed = true;
                } else {
                    self.song.tempo = ch.eff as u16;
                    self.song.timer = ch.eff as u16;
                }
            }

            // Gxx - set global volume
            else if ch.eff_typ == 16 {
                self.song.glob_vol = ch.eff as u16;
                if self.song.glob_vol > 64 {
                    self.song.glob_vol = 64;
                }

                set_global_volume = true;
            }

            // Lxx - set vol and pan envelope position
            else if ch.eff_typ == 21 {
                // *** VOLUME ENVELOPE ***
                if ins.env_v_typ & 1 != 0 {
                    ch.env_v_cnt = ch.eff as u16 - 1;

                    let mut env_pos = 0;
                    let mut env_update = true;
                    let mut new_env_pos = ch.eff as i16;

                    if ins.env_vp_ant > 1 {
                        env_pos += 1;
                        for i in 0..ins.env_vp_ant {
                            if new_env_pos < ins.env_vp[env_pos][0] {
                                env_pos -= 1;

                                new_env_pos -= ins.env_vp[env_pos][0];
                                if new_env_pos == 0 {
                                    env_update = false;
                                    break
                                }

                                if ins.env_vp[env_pos + 1][0] <= ins.env_vp[env_pos][0] {
                                    env_update = true;
                                    break
                                }

                                ch.env_v_ip_value = ((ins.env_vp[env_pos + 1][1] - ins.env_vp[env_pos][1]) & 0x00FF) << 8;
                                ch.env_v_ip_value /= ins.env_vp[env_pos + 1][0] - ins.env_vp[env_pos][0];

                                ch.env_v_amp = (ch.env_v_ip_value * (new_env_pos - 1) + (ins.env_vp[env_pos][1] & 0x00FF) << 8) as u16;

                                env_pos += 1;

                                env_update = false;
                                break
                            }

                            env_pos += 1;
                        }

                        if env_update {
                            env_pos -= 1;
                        }
                    }

                    if env_update {
                        ch.env_v_ip_value = 0;
                        ch.env_v_amp = ((ins.env_vp[env_pos][1] & 0x00FF) << 8) as u16;
                    }

                    if env_pos >= ins.env_vp_ant as usize {
                        env_pos = ins.env_vp_ant as usize - 1;
                        if env_pos < 0 {
                            env_pos = 0;
                        }
                    }

                    ch.env_v_pos = env_pos as u8;
                }

                // *** PANNING ENVELOPE ***
                if ins.env_v_typ & 2 != 0 {  // probably an FT2 bug
                    ch.env_p_cnt = ch.eff as u16 - 1;

                    let mut env_pos = 0;
                    let mut env_update = true;
                    let mut new_env_pos = ch.eff as i16;

                    if ins.env_pp_ant > 1 {
                        env_pos += 1;
                        for i in 0..ins.env_pp_ant - 1 {
                            if new_env_pos < ins.env_pp[env_pos][0] {
                                env_pos -= 1;

                                new_env_pos -= ins.env_pp[env_pos][0];
                                if new_env_pos == 0 {
                                    env_update = false;
                                    break
                                }

                                if ins.env_pp[env_pos + 1][0] <= ins.env_pp[env_pos][0] {
                                    env_update = true;
                                    break
                                }

                                ch.env_p_ip_value = ((ins.env_pp[env_pos + 1][1] - ins.env_pp[env_pos][1]) & 0x00FF) << 8;
                                ch.env_p_ip_value /= ins.env_pp[env_pos + 1][0] - ins.env_pp[env_pos][0];

                                ch.env_p_amp = ((ch.env_p_ip_value * (new_env_pos - 1)) + (ins.env_pp[env_pos][1] & 0x00FF) << 8) as u16;

                                env_pos += 1;

                                env_update = false;
                                break
                            }

                            env_pos += 1;
                        }

                        if env_update {
                            env_pos -= 1;
                        }
                    }

                    if env_update {
                        ch.env_p_ip_value = 0;
                        ch.env_p_amp = ((ins.env_pp[env_pos][1] & 0x00FF) << 8) as u16;
                    }

                    if env_pos >= ins.env_pp_ant as usize {
                        env_pos = ins.env_pp_ant as usize - 1;
                        if env_pos < 0 {
                            env_pos = 0;
                        }
                    }

                    ch.env_p_pos = env_pos as u8;
                }
            }
        }
        if set_speed {
            let speed = self.song.speed;
            self.set_speed(speed);
        }
        if set_global_volume {
            for i in 0..self.song.ant_chn as usize {
                self.stm[i].status |= IS_VOL;
            }
        }
    }

    fn check_effects(&mut self, chn: usize) {

        let mut multi_retrig = false;
        {
            let ch = &mut self.stm[chn];

            // this one is manipulated by vol column effects, then used for multiretrig (FT2 quirk)
            let mut vol_kol = ch.vol_kol_vol;

            // *** VOLUME COLUMN EFFECTS (TICK 0) ***

            // set volume
            if ch.vol_kol_vol >= 0x10 && ch.vol_kol_vol <= 0x50 {
                vol_kol -= 16;

                ch.out_vol  = vol_kol as i8;
                ch.real_vol = vol_kol as i8;

                ch.status |= IS_VOL + IS_QUICKVOL;
            }

            // fine volume slide down
            else if ch.vol_kol_vol & 0xF0 == 0x80 {
                vol_kol = ch.vol_kol_vol & 0x0F;

                // unsigned clamp
                if ch.real_vol >= vol_kol as i8 {
                    ch.real_vol -= vol_kol as i8;
                } else {
                    ch.real_vol = 0;
                }

                ch.out_vol = ch.real_vol;
                ch.status |= IS_VOL;
            }

            // fine volume slide up
            else if ch.vol_kol_vol & 0xF0 == 0x90 {
                vol_kol = ch.vol_kol_vol & 0x0F;

                // unsigned clamp
                if ch.real_vol <= 64 - vol_kol as i8 {
                    ch.real_vol += vol_kol as i8;
                } else {
                    ch.real_vol = 64;
                }

                ch.out_vol = ch.real_vol;
                ch.status |= IS_VOL;
            }

            // set vibrato speed
            else if ch.vol_kol_vol & 0xF0 == 0xA0 {
                vol_kol = (ch.vol_kol_vol & 0x0F) << 2;
                ch.vib_speed = vol_kol;
            }

            // set panning
            else if ch.vol_kol_vol & 0xF0 == 0xC0 {
                vol_kol <<= 4;

                ch.out_pan = vol_kol;
                ch.status |= IS_PAN;
            }


            // *** MAIN EFFECTS (TICK 0) ***


            if ch.eff_typ == 0 && ch.eff == 0 {
                return
            }

            // Cxx - set volume
            if ch.eff_typ == 12 {
                ch.real_vol = ch.eff as i8;
                if ch.real_vol > 64 {
                    ch.real_vol = 64
                }

                ch.out_vol = ch.real_vol;
                ch.status |= IS_VOL + IS_QUICKVOL;

                return;
            }

            // 8xx - set panning
            else if ch.eff_typ == 8 {
                ch.out_pan = ch.eff;
                ch.status |= IS_PAN;

                return
            }

            // Rxy - note multi retrigger
            else if ch.eff_typ == 27 {
                let mut tmp_eff = ch.eff & 0x0F;
                if tmp_eff == 0 {
                    tmp_eff = ch.retrig_speed;
                }

                ch.retrig_speed = tmp_eff;

                let mut tmp_eff_hi = ch.eff >> 4;
                if tmp_eff_hi == 0 {
                    tmp_eff_hi = ch.retrig_vol;
                }

                ch.retrig_vol = tmp_eff_hi;

                if vol_kol == 0 {
                    multi_retrig = true;
                } else {
                    return
                }
            }

            // X1x - extra fine period slide up
            else if ch.eff_typ == 33 && ch.eff & 0xF0 == 0x10 {
                let mut tmp_eff = ch.eff & 0x0F;
                if tmp_eff == 0 {
                    tmp_eff = ch.e_porta_up_speed
                }

                ch.e_porta_up_speed = tmp_eff;

                ch.real_period -= tmp_eff as i16;
                if ch.real_period < 1 {
                     ch.real_period = 1
                }

                ch.out_period = ch.real_period as u16;
                ch.status |= IS_PERIOD;

                return
            }

            // X2x - extra fine period slide down
            else if ch.eff_typ == 33 && ch.eff & 0xF0 == 0x20 {
                let mut tmp_eff = ch.eff & 0x0F;
                if tmp_eff == 0 {
                    tmp_eff = ch.e_porta_down_speed
                }

                ch.e_porta_down_speed = tmp_eff;

                ch.real_period += tmp_eff as i16;
                if ch.real_period > 32000 - 1 {
                    ch.real_period = 32000 - 1
                }

                ch.out_period = ch.real_period as u16;
                ch.status |= IS_PERIOD;

                return
            }
        }

        if multi_retrig {
            self.multi_retrig(chn);
            return
        }

        self.check_more_effects(chn);
    }

    fn fix_tone_porta(&mut self, chn: usize, p: &TonTyp, inst: u8) {


        if p.ton != 0 {
            if p.ton == 97 {
                self.key_off(chn);
            } else {
                let ch = &mut self.stm[chn];

                /* MUST use >> 3 here (sar cl,3) - safe for x86/x86_64 */
                let porta_tmp = ((((p.ton as i8 - 1) + ch.rel_ton_nr) & 0x00FF) * 16) + ((ch.fine_tune >> 3) + 16) & 0x00FF;

                if porta_tmp < MAX_NOTES as i8 {
                    ch.want_period = self.note2period[porta_tmp as usize] as u16;

                    if ch.want_period == ch.real_period as u16 {
                        ch.porta_dir = 0
                    } else if ch.want_period > ch.real_period as u16 {
                        ch.porta_dir = 1;
                    } else {
                        ch.porta_dir = 2;
                    }
                }
            }
        }

        if inst != 0 {
            self.retrig_volume(chn);

            if p.ton != 97 {
                self.retrig_envelope_vibrato(chn);
            }
        }
    }

}


impl<'a> FormatPlayer for Ft2Play<'a> {
    fn start(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<XmData>().unwrap();


        //data.speed = self.song.speed as usize;
        //data.tempo = self.song.tempo as usize;
    }

    fn play(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<XmData>().unwrap();

        //self.dorow(&module, &mut mixer);

        /*data.frame = self.musiccount as usize;
        data.row = self.np_row as usize;
        data.pos = self.np_ord as usize - 1;

        data.speed = self.musicmax as usize;
        data.tempo = self.tempo as usize;*/
    }

    fn reset(&mut self) {
    }
}

