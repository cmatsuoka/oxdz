use std::cmp;
use std::f64::consts::PI;
use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer, State};
use player::scan::SaveRestore;
use format::xm::{XmData, TonTyp};
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
    //song_tab     : Vec<u8>,
    ver            : u16,
    name           : String,
}

/*
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
    //mute       : bool,
    samp       : [SampleTyp; 32],
}
*/

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
    want_period           : i16, //u16,
    final_period          : u16,
    out_period            : u16,
    fade_out_amp          : u32,

    smp_ptr               : usize,  // oxdz: store these as indexes instead of ponters
    instr_ptr             : usize,
}


const MAX_NOTES : u16 = (12 * 10 * 16) + 16;
const MAX_VOICES: usize = 32;


// TABLES AND VARIABLES
lazy_static! {
    static ref PANNING_TAB: Box<[u32; 257]> = Box::new([
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
    ]);

    static ref AMIGA_FINE_PERIOD: Box<[u16; 12 * 8]> = Box::new([
        907, 900, 894, 887, 881, 875, 868, 862, 856, 850, 844, 838,
        832, 826, 820, 814, 808, 802, 796, 791, 785, 779, 774, 768,
        762, 757, 752, 746, 741, 736, 730, 725, 720, 715, 709, 704,
        699, 694, 689, 684, 678, 675, 670, 665, 660, 655, 651, 646,
        640, 636, 632, 628, 623, 619, 614, 610, 604, 601, 597, 592,
        588, 584, 580, 575, 570, 567, 563, 559, 555, 551, 547, 543,
        538, 535, 532, 528, 524, 520, 516, 513, 508, 505, 502, 498,
        494, 491, 487, 484, 480, 477, 474, 470, 467, 463, 460, 457
    ]);

    static ref VIB_TAB: Box<[u8; 32]> = Box::new([
        0,   24,   49,  74,  97, 120, 141, 161,
        180, 197, 212, 224, 235, 244, 250, 253,
        255, 253, 250, 244, 235, 224, 212, 197,
        180, 161, 141, 120,  97,  74,  49,  24
    ]);
}


#[derive(Default, SaveRestore)]
pub struct Ft2Play {
    linear_frq_tab      : bool,
    patt_lens           : Vec<u16>,
    speed_val           : u32,
    real_replay_rate    : u32,
    f_audio_freq        : f32,
    quick_vol_ramp_mul_f: f32,
    tick_vol_ramp_mul_f : f32,
    song                : SongTyp,
    stm                 : [StmTyp; MAX_VOICES],
    //instr             : Vec<InstrTyp>,

    vib_sine_tab        : Vec<i8>,
    //linear_periods      : Vec<i16>,
    //amiga_periods       : Vec<i16>,
    note2period         : Vec<i16>,
    log_tab             : Vec<u32>,
}

impl Ft2Play {
    pub fn new(_module: &Module, _options: Options) -> Self {
        let mut ft2: Ft2Play = Default::default();
        ft2.patt_lens = Vec::new();
        //ft2.song.song_tab = Vec::new();
        ft2
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

    fn retrig_envelope_vibrato(&mut self, chn: usize, module: &XmData) {
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

        //let ins = &self.instr[ch.instr_ptr];
        let ins = &module.instruments[ch.instr_ptr];

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

    fn key_off(&mut self, chn: usize, module: &XmData) {
        let ch = &mut self.stm[chn];

        ch.env_sustain_active = false;

        //let ins = &self.instr[ch.instr_ptr];
        let ins = &module.instruments[ch.instr_ptr];

        // oxdz: we don't allocate empty data
        if ins.ant_samp == 0 {
            return;
        }

        if ins.env_p_typ & 1 == 0 {  // yes, FT2 does this (!)
            if ch.env_p_cnt >= ins.env_pp[ch.env_p_pos as usize].0 as u16 {
                ch.env_p_cnt = (ins.env_pp[ch.env_p_pos as usize].0 - 1) as u16;
            }
        }

        if ins.env_v_typ & 1 != 0 {
            if ch.env_v_cnt >= ins.env_vp[ch.env_v_pos as usize].0 as u16 {
                ch.env_v_cnt = (ins.env_vp[ch.env_v_pos as usize].0 - 1) as u16;
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

    fn start_tone(&mut self, mut ton: u8, eff_typ: u8, eff: u8, chn: usize, module: &XmData) {

        // no idea if this EVER triggers...
        if ton == 97 {
            self.key_off(chn, &module);
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

        // oxdz: sanity check
        if ch.instr_nr < 1 || ch.instr_nr >= module.header.ant_instrs {
            return
        }

        //let ins = &self.instr[ch.instr_nr as usize];
        let ins = &module.instruments[ch.instr_nr as usize - 1];

        // oxdz: sanity check
        if ins.ant_samp == 0 {
            return
        }

        //ch.instr_ptr = ins;
        ch.instr_ptr = ch.instr_nr as usize - 1;

        //ch.mute = ins.mute;

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
            (((eff & 0x0F) * 16) - 128) as i8  // result is now -128 .. 127
        } else {
            s.fine
        };

        if ton > 0 {
            // MUST use >> 3 here (sar cl,3) - safe for x86/x86_64
            let tmp_ton = ((ton - 1) as u16 * 16) + ((((ch.fine_tune >> 3) + 16) as u16) & 0xFF);

            if tmp_ton < MAX_NOTES {  // should never happen, but FT2 does this check
                ch.real_period = self.note2period[tmp_ton as usize];
                ch.out_period  = ch.real_period as u16;
            }
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

    fn multi_retrig(&mut self, chn: usize, module: &XmData) {
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

        self.start_tone(0, 0, 0, chn, &module);
    }

    fn check_more_effects(&mut self, chn: usize, module: &XmData) {
        let mut set_speed = false;
        let mut set_global_volume = false;
        {
            let ch = &mut self.stm[chn];
            //let ins = &self.instr[ch.instr_ptr];
            let ins = &module.instruments[ch.instr_ptr];

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
                        for _i in 0..ins.env_vp_ant {
                            if new_env_pos < ins.env_vp[env_pos].0 {
                                env_pos -= 1;

                                new_env_pos -= ins.env_vp[env_pos].0;
                                if new_env_pos == 0 {
                                    env_update = false;
                                    break
                                }

                                if ins.env_vp[env_pos + 1].0 <= ins.env_vp[env_pos].0 {
                                    env_update = true;
                                    break
                                }

                                ch.env_v_ip_value = ((ins.env_vp[env_pos + 1].1 - ins.env_vp[env_pos].1) & 0x00FF) << 8;
                                ch.env_v_ip_value /= ins.env_vp[env_pos + 1].0 - ins.env_vp[env_pos].0;

                                ch.env_v_amp = (ch.env_v_ip_value * (new_env_pos - 1) + (ins.env_vp[env_pos].1 & 0x00FF) << 8) as u16;

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
                        ch.env_v_amp = ((ins.env_vp[env_pos].1 & 0x00FF) << 8) as u16;
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
                        for _i in 0..ins.env_pp_ant - 1 {
                            if new_env_pos < ins.env_pp[env_pos].0 {
                                env_pos -= 1;

                                new_env_pos -= ins.env_pp[env_pos].0;
                                if new_env_pos == 0 {
                                    env_update = false;
                                    break
                                }

                                if ins.env_pp[env_pos + 1].0 <= ins.env_pp[env_pos].0 {
                                    env_update = true;
                                    break
                                }

                                ch.env_p_ip_value = ((ins.env_pp[env_pos + 1].1 - ins.env_pp[env_pos].1) & 0x00FF) << 8;
                                ch.env_p_ip_value /= ins.env_pp[env_pos + 1].0 - ins.env_pp[env_pos].0;

                                ch.env_p_amp = ((ch.env_p_ip_value * (new_env_pos - 1)) + (ins.env_pp[env_pos].1 & 0x00FF) << 8) as u16;

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
                        ch.env_p_amp = ((ins.env_pp[env_pos].1 & 0x00FF) << 8) as u16;
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

    fn check_effects(&mut self, chn: usize, module: &XmData) {

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
            self.multi_retrig(chn, &module);
            return
        }

        self.check_more_effects(chn, &module);
    }

    fn fix_tone_porta(&mut self, chn: usize, p: &TonTyp, inst: u8, module: &XmData) {

        if p.ton != 0 {
            if p.ton == 97 {
                self.key_off(chn, &module);
            } else {
                let ch = &mut self.stm[chn];

                // MUST use >> 3 here (sar cl,3) - safe for x86/x86_64
                let porta_tmp = ((((p.ton as i16 - 1) + ch.rel_ton_nr as i16) & 0x00FF) as u16 * 16) + ((((ch.fine_tune >> 3) + 16) as u16) & 0x00FF);

                if porta_tmp < MAX_NOTES {
                    ch.want_period = self.note2period[porta_tmp as usize];

                    if ch.want_period == ch.real_period {
                        ch.porta_dir = 0
                    } else if ch.want_period > ch.real_period {
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
                self.retrig_envelope_vibrato(chn, &module);
            }
        }
    }

    fn get_new_note(&mut self, chn: usize, p: &TonTyp, module: &XmData) {
        // this is a mess, but it appears to be 100% FT2-correct

        {
            let ch = &mut self.stm[chn];

            ch.vol_kol_vol = p.vol;

            if ch.eff_typ == 0 {
                if ch.eff != 0 {
                    // we have an arpeggio running, set period back
                    ch.out_period = ch.real_period as u16;
                    ch.status |= IS_PERIOD;
                }
            } else {
                if ch.eff_typ == 4 || ch.eff_typ == 6 {
                    // we have a vibrato running
                    if p.eff_typ != 4 && p.eff_typ != 6 {
                        // but it's ending at the next (this) row, so set period back
                        ch.out_period = ch.real_period as u16;
                        ch.status |= IS_PERIOD;
                    }
                }
            }

            ch.eff_typ = p.eff_typ;
            ch.eff     = p.eff;
            ch.ton_typ = ((p.instr as u16) << 8) | p.ton as u16;
        }

        // 'inst' var is used for later if checks...
        let mut inst = p.instr;
        if inst != 0 {
            if inst <= 128 {
                self.stm[chn].instr_nr = inst as u16
            } else {
                inst = 0
            }
        }

        let mut check_efx = true;
        if p.eff_typ == 0x0E {
            if p.eff >= 0xD1 && p.eff <= 0xDF {
                return  // we have a note delay (ED1..EDF)
            } else if p.eff == 0x90 {
                check_efx = false
            }
        }

        if check_efx {
            if self.stm[chn].vol_kol_vol & 0xF0 == 0xF0 {  // gxx
                if self.stm[chn].vol_kol_vol & 0x0F != 0 {
                    self.stm[chn].porta_speed = ((self.stm[chn].vol_kol_vol & 0x0F) as u16) << 6;
                }

                self.fix_tone_porta(chn, p, inst, &module);
                self.check_effects(chn, &module);
                return
            }

            if p.eff_typ == 3 || p.eff_typ == 5 {  // 3xx or 5xx
                if p.eff_typ != 5 && p.eff != 0 {
                    self.stm[chn].porta_speed = (p.eff as u16) << 2;
                }

                self.fix_tone_porta(chn, p, inst, &module);
                self.check_effects(chn, &module);
                return
            }

            if p.eff_typ == 0x14 && p.eff == 0 {  // K00 (KeyOff - only handle tick 0 here)
                self.key_off(chn, &module);

                if inst != 0 {
                    self.retrig_volume(chn);
                }

                self.check_effects(chn, &module);
                return
            }

            if p.ton == 0 {
                if inst != 0 {
                    self.retrig_volume(chn);
                    self.retrig_envelope_vibrato(chn, &module);
                }

                self.check_effects(chn, &module);
                return
            }
        }

        if p.ton == 97 {
            self.key_off(chn, &module)
        } else {
            self.start_tone(p.ton, p.eff_typ, p.eff, chn, &module)
        }

        if inst != 0 {
            self.retrig_volume(chn);

            if p.ton != 97 {
                self.retrig_envelope_vibrato(chn, &module);
            }
        }

        self.check_effects(chn, &module);
    }

    fn fixa_envelope_vibrato(&mut self, chn: usize, module: &XmData) {
/*
        int8_t env_interpolate_flag, env_did_interpolate;
        uint8_t env_pos;
        int16_t auto_vib_val, pan_tmp;
        uint16_t auto_vib_amp, tmp_period, env_val;
        instrTyp *ins;
*/

        let ch = &mut self.stm[chn];
        //let ins = &self.instr[ch.instr_ptr];
        let ins = &module.instruments[ch.instr_ptr];

        // *** FADEOUT ***
        if !ch.env_sustain_active {
            ch.status |= IS_VOL;

            // unsigned clamp + reset
            if ch.fade_out_amp >= ch.fade_out_speed as u32 {
                ch.fade_out_amp -= ch.fade_out_speed as u32;
            } else {
                ch.fade_out_amp = 0;
                ch.fade_out_speed = 0;
            }
        }

        if !ch.mute {
            // *** VOLUME ENVELOPE ***
            let mut env_val = 0;
            if ins.env_v_typ & 1 != 0 {
                let mut env_did_interpolate = false;
                let mut env_pos = ch.env_v_pos as usize;

                ch.env_v_cnt = ch.env_v_cnt.wrapping_add(1);
                if ch.env_v_cnt == ins.env_vp[env_pos].0 as u16 {
                    ch.env_v_amp = ((ins.env_vp[env_pos].1 & 0x00FF) as u16) << 8;

                    env_pos += 1;
                    if ins.env_v_typ & 4 != 0 {  // envelope loop
                        env_pos -= 1;

                        if env_pos == ins.env_v_rep_e as usize {
                            if ins.env_v_typ & 2 == 0 || env_pos != ins.env_v_sust as usize || ch.env_sustain_active {
                                env_pos = ins.env_v_rep_s as usize;

                                ch.env_v_cnt = ins.env_vp[env_pos].0 as u16;
                                ch.env_v_amp = ((ins.env_vp[env_pos].1 & 0x00FF) as u16) << 8;
                            }
                        }

                        env_pos += 1;
                    }

                    if env_pos < ins.env_vp_ant as usize {
                        let mut env_interpolate_flag = true;
                        if ins.env_v_typ & 2 != 0 && ch.env_sustain_active {
                            if env_pos - 1 == ins.env_v_sust as usize {
                                env_pos -= 1;
                                ch.env_v_ip_value = 0;
                                env_interpolate_flag = false;
                            }
                        }

                        if env_interpolate_flag {
                            ch.env_v_pos = env_pos as u8;

                            ch.env_v_ip_value = 0;
                            if ins.env_vp[env_pos].0 > ins.env_vp[env_pos - 1].0 {
                                ch.env_v_ip_value = ((ins.env_vp[env_pos].1 - ins.env_vp[env_pos - 1].1) & 0x00FF) << 8;
                                ch.env_v_ip_value /= ins.env_vp[env_pos].0 - ins.env_vp[env_pos - 1].0;

                                env_val = ch.env_v_amp;
                                env_did_interpolate = true;
                            }
                        }
                    } else {
                        ch.env_v_ip_value = 0;
                    }
                }

                if !env_did_interpolate {
                    ch.env_v_amp = (ch.env_v_amp as i32 + ch.env_v_ip_value as i32) as u16;

                    env_val = ch.env_v_amp as u16;
                    if (env_val>>8) > 0x40 {
                        if (env_val>>8) > (0x40 + 0xC0) / 2 {
                            env_val = 16384;
                        } else {
                            env_val = 0;
                        }

                        ch.env_v_ip_value = 0;
                    }
                }

                env_val >>= 8;

                ch.final_vol = ((self.song.glob_vol as u32 * (((env_val * ch.out_vol as u16) as u32 * ch.fade_out_amp) >> (16 + 2))) >> 7) as u16;
                ch.status  |= IS_VOL;
            } else {
                ch.final_vol = ((self.song.glob_vol as u32 * ((((ch.out_vol as u32) << 4) * ch.fade_out_amp) >> 16)) >> 7) as u16;
            }

            // non-FT2 ear security system
            if ch.final_vol > 256 {
                ch.final_vol = 256;
            }
        } else {
            ch.final_vol = 0;
        }

        // *** PANNING ENVELOPE ***

        let mut env_val: i32 = 0;
        if ins.env_p_typ & 1 != 0 {
            let mut env_did_interpolate = false;
            let mut env_pos = ch.env_p_pos as usize;

            ch.env_p_cnt = ch.env_p_cnt.wrapping_add(1);
            if ch.env_p_cnt == ins.env_pp[env_pos].0 as u16 {
                ch.env_p_amp = ((ins.env_pp[env_pos].1 & 0x00FF) as u16) << 8;

                env_pos += 1;
                if ins.env_p_typ & 4 != 0 {
                    env_pos -= 1;

                    if env_pos == ins.env_p_rep_e as usize {
                        if ins.env_p_typ & 2 == 0 || env_pos != ins.env_p_sust as usize || ch.env_sustain_active {
                            env_pos = ins.env_p_rep_s as usize;

                            ch.env_p_cnt = ins.env_pp[env_pos].0 as u16;
                            ch.env_p_amp = ((ins.env_pp[env_pos].1 & 0x00FF) as u16) << 8;
                        }
                    }

                    env_pos += 1;
                }

                if env_pos < ins.env_pp_ant as usize {
                    let mut env_interpolate_flag = true;
                    if ins.env_p_typ&2 != 0 && ch.env_sustain_active {
                        if env_pos - 1 == ins.env_p_sust as usize {
                            env_pos -= 1;
                            ch.env_p_ip_value = 0;
                            env_interpolate_flag = false;
                        }
                    }

                    if env_interpolate_flag {
                        ch.env_p_pos = env_pos as u8;

                        ch.env_p_ip_value = 0;
                        if ins.env_pp[env_pos].0 > ins.env_pp[env_pos - 1].0 {
                            ch.env_p_ip_value  = ((ins.env_pp[env_pos].1 - ins.env_pp[env_pos - 1].1) & 0x00FF) << 8;
                            ch.env_p_ip_value /= ins.env_pp[env_pos].0 - ins.env_pp[env_pos - 1].0;

                            env_val = ch.env_p_amp as i32;
                            env_did_interpolate = true;
                        }
                    }
                } else {
                    ch.env_p_ip_value = 0;
                }
            }

            if !env_did_interpolate {
                ch.env_p_amp += ch.env_p_ip_value as u16;

                env_val = ch.env_p_amp as i32;
                if env_val>>8 > 0x40 {
                    if env_val>>8 > (0x40 + 0xC0) / 2 {
                        env_val = 16384;
                    } else {
                        env_val = 0;
                    }

                    ch.env_p_ip_value = 0;
                }
            }

            // panning calculated exactly like FT2
            let mut pan_tmp = ch.out_pan.wrapping_sub(128) as i8;
            if pan_tmp > 0 {
                pan_tmp = -pan_tmp;
            }
            //pan_tmp = pan_tmp.wrapping_add(128);
            pan_tmp = pan_tmp.wrapping_sub(1);

            env_val -= 32 * 256;

            ch.final_pan = (ch.out_pan as i32 + (((env_val as i32 * ((pan_tmp as i32) << 3)) >> 16) & 0xFF)) as u8;
            ch.status  |= IS_PAN;
        } else {
            ch.final_pan = ch.out_pan;
        }

        // *** AUTO VIBRATO ***
        if ins.vib_depth != 0 {
            let mut auto_vib_amp: u16;
            let mut auto_vib_val: i32;

            if ch.e_vib_sweep != 0 {
                auto_vib_amp = ch.e_vib_sweep;
                if ch.env_sustain_active {
                    auto_vib_amp += ch.e_vib_amp;
                    if auto_vib_amp>>8 > ins.vib_depth as u16 {
                        auto_vib_amp = (ins.vib_depth as u16) << 8;
                        ch.e_vib_sweep = 0;
                    }

                    ch.e_vib_amp = auto_vib_amp;
                }
            } else {
                auto_vib_amp = ch.e_vib_amp;
            }

            ch.e_vib_pos = ch.e_vib_pos.wrapping_add(ins.vib_rate);

            // square
            if ins.vib_typ == 1 {
                auto_vib_val = if ch.e_vib_pos > 127 { 64 } else { -64 };
            }

            // ramp up
            else if ins.vib_typ == 2 {
                auto_vib_val = ((((ch.e_vib_pos as i32) >> 1) + 64) & 127) - 64;
            }

            // ramp down
            else if ins.vib_typ == 3 {
                auto_vib_val = (((0 - ((ch.e_vib_pos as i32) >> 1)) + 64) & 127) - 64;
            }

            // sine
            else {
                auto_vib_val = self.vib_sine_tab[ch.e_vib_pos as usize] as i32;
            }

            auto_vib_val <<= 2;

            let mut tmp_period = ch.out_period + ((auto_vib_val * auto_vib_amp as i32) >> 16) as u16;
            if tmp_period > (32000 - 1) {
                tmp_period = 0; // yes, FT2 zeroes it out
            }

            ch.final_period = tmp_period;

            ch.status  |= IS_PERIOD;
        } else {
            ch.final_period = ch.out_period;
        }
    }

    fn relocate_ton(&mut self, period: i16, relative_note: i8, chn: usize) -> i16 {
        let ch = &mut self.stm[chn];

        let fine_tune: i32  = ((ch.fine_tune as i32) >> 3) + 16;  // MUST use >> 3 here (sar cl,3) - safe for x86/x86_64
        let mut hi_period: i32 = 8 * 12 * 16;
        let mut lo_period: i32 = 0;

        let period_table = &self.note2period;

        for _i in 0..8 {
            let tmp_period = (((lo_period + hi_period) / 2) & !15) + fine_tune;

            let mut table_index = tmp_period as i32 - 8;
            if table_index < 0 {  // added security check
                table_index = 0
            }

            if period >= period_table[table_index as usize] {
                hi_period = tmp_period - fine_tune;
            } else {
                lo_period = tmp_period - fine_tune;
            }
        }

        let mut tmp_period = lo_period + fine_tune + (relative_note as i32 * 16);
        if tmp_period < 0 {  // added security check
            tmp_period = 0
        }

        if tmp_period >= ((8 * 12 * 16) + 15) - 1 {  // FT2 bug: stupid off-by-one edge case
            tmp_period  =  (8 * 12 * 16) + 15
        }

        return period_table[tmp_period as usize];
    }

    fn tone_porta(&mut self, chn: usize) {

        if self.stm[chn].porta_dir != 0 {
            if self.stm[chn].porta_dir > 1 {
                let ch = &mut self.stm[chn];
                ch.real_period -= ch.porta_speed as i16;
                if ch.real_period <= ch.want_period {
                    ch.porta_dir   = 1;
                    ch.real_period = ch.want_period;
                }
            } else {
                let ch = &mut self.stm[chn];
                ch.real_period += ch.porta_speed as i16;
                if ch.real_period >= ch.want_period {
                    ch.porta_dir   = 1;
                    ch.real_period = ch.want_period;
                }
            }

            if self.stm[chn].gliss_funk != 0 {  // semi-tone slide flag
                let period = self.stm[chn].real_period;
                self.stm[chn].out_period = self.relocate_ton(period, 0, chn) as u16;
            } else {
                self.stm[chn].out_period = self.stm[chn].real_period as u16;
            }

            self.stm[chn].status |= IS_PERIOD;
        }
    }

    fn volume(&mut self, chn: usize) {  // actually volume slide
        let ch = &mut self.stm[chn];

        let mut tmp_eff = ch.eff;
        if tmp_eff == 0 {
            tmp_eff = ch.vol_slide_speed;
        }

        ch.vol_slide_speed = tmp_eff;

        if tmp_eff & 0xF0 == 0 {
            // unsigned clamp
            if ch.real_vol >= tmp_eff as i8 {
                ch.real_vol -= tmp_eff as i8;
            } else {
                ch.real_vol = 0;
            }
        } else {
            // unsigned clamp
            if ch.real_vol <= 64 - (tmp_eff >> 4) as i8 {
                ch.real_vol += (tmp_eff >> 4) as i8;
            } else {
                ch.real_vol = 64;
            }
        }

        ch.out_vol = ch.real_vol;
        ch.status |= IS_VOL;
    }

    fn vibrato2(&mut self, chn: usize) {
        let ch = &mut self.stm[chn];

        let mut tmp_vib = (ch.vib_pos / 4) & 0x1F;

        match ch.wave_ctrl & 0x03 {
            // 0: sine
            0 => tmp_vib = VIB_TAB[tmp_vib as usize],

            // 1: ramp
            1 => {
                tmp_vib *= 8;
                if ch.vib_pos >= 128 {
                    tmp_vib ^= 0xFF;
                }
            }

            // 2/3: square
            _ => tmp_vib = 255,
        }

        tmp_vib = ((tmp_vib as u16 * ch.vib_depth as u16) / 32) as u8;

        ch.out_period = if ch.vib_pos >= 128 {
            ch.real_period - tmp_vib as i16
        } else {
            ch.real_period + tmp_vib as i16
        } as u16;

        ch.status |= IS_PERIOD;
        ch.vib_pos = ch.vib_pos.wrapping_add(ch.vib_speed);
    }

    fn vibrato(&mut self, chn: usize) {
        {
            let ch = &mut self.stm[chn];

            if ch.eff != 0 {
                if ch.eff & 0x0F != 0 {
                    ch.vib_depth = ch.eff & 0x0F;
                }
                if ch.eff & 0xF0 != 0 {
                    ch.vib_speed = (ch.eff >> 4) * 4;
                }
            }
        }

        self.vibrato2(chn);
    }

    fn do_effects(&mut self, chn: usize, module: &XmData) {

        // *** VOLUME COLUMN EFFECTS (TICKS >0) ***

        let vol_kol_vol = self.stm[chn].vol_kol_vol;

        // volume slide down
        if vol_kol_vol & 0xF0 == 0x60 {
            let ch = &mut self.stm[chn];

            // unsigned clamp
            if ch.real_vol >= (vol_kol_vol & 0x0F) as i8 {
                ch.real_vol -= (vol_kol_vol & 0x0F) as i8;
            } else {
                ch.real_vol = 0;
            }

            ch.out_vol = ch.real_vol;
            ch.status |= IS_VOL;
        }

        // volume slide up
        else if vol_kol_vol & 0xF0 == 0x7 {
            let ch = &mut self.stm[chn];

            // unsigned clamp
            if ch.real_vol <= 64 - (vol_kol_vol & 0x0F) as i8 {
                ch.real_vol += (vol_kol_vol & 0x0F) as i8;
            } else {
                ch.real_vol = 64;
            }

            ch.out_vol = ch.real_vol;
            ch.status |= IS_VOL;
        }

        // vibrato (+ set vibrato depth)
        else if vol_kol_vol & 0xF0 == 0xB0 {
            if vol_kol_vol != 0xB0 {
                self.stm[chn].vib_depth = vol_kol_vol & 0x0F;
            }

            self.vibrato2(chn);
        }

        // pan slide left
        else if vol_kol_vol & 0xF0 == 0xD0 {
            let ch = &mut self.stm[chn];

            // unsigned clamp + a bug when the parameter is 0
            if vol_kol_vol & 0x0F == 0 || ch.out_pan < vol_kol_vol & 0x0F {
                ch.out_pan = 0;
            } else {
                ch.out_pan -= vol_kol_vol & 0x0F;
            }

            ch.status |= IS_PAN;
        }

        // pan slide right
        else if vol_kol_vol & 0xF0 == 0xE0 {
            let ch = &mut self.stm[chn];

            // unsigned clamp
            if ch.out_pan <= 255 - (vol_kol_vol & 0x0F) {
                ch.out_pan += vol_kol_vol & 0x0F;
            } else {
                ch.out_pan = 255;
            }

            ch.status |= IS_PAN;
        }

        // tone portamento
        else if vol_kol_vol & 0xF0 == 0xF0 {
            self.tone_porta(chn);
        }

        // *** MAIN EFFECTS (TICKS >0) ***

        let eff = self.stm[chn].eff;
        let eff_typ = self.stm[chn].eff_typ;

        if (eff == 0 && eff_typ == 0) || eff_typ >= 36 {
            return
        }

        // 0xy - Arpeggio
        if eff_typ == 0 {
            let mut tick = self.song.timer;
            let mut note = 0;

            // FT2 'out of boundary' arp LUT simulation
            if tick > 16 {
                tick = 2;
            } else if tick == 16 {
                tick = 0;
            } else {
                tick %= 3;
            }

            //
            // this simulation doesn't work properly for >=128 tick arps,
            // but you'd need to hexedit the initial speed to get >31
            //

            self.stm[chn].out_period = if tick == 0 {
                self.stm[chn].real_period
            } else {
                if tick == 1 {
                    note = eff >> 4;
                } else if tick > 1 {
                    note = eff & 0x0F;
                }

                let period = self.stm[chn].real_period;
                self.relocate_ton(period, note as i8, chn)
            } as u16;

            self.stm[chn].status |= IS_PERIOD;
        }

        // 1xx - period slide up
        else if eff_typ == 1 {
            let ch = &mut self.stm[chn];

            let mut tmp_eff = eff;
            if tmp_eff == 0 {
                tmp_eff = ch.porta_up_speed;
            }

            ch.porta_up_speed = tmp_eff;

            ch.real_period -= tmp_eff as i16 * 4;
            if ch.real_period < 1 {
                ch.real_period = 1;
            }

            ch.out_period = ch.real_period as u16;
            ch.status |= IS_PERIOD;
        }

        // 2xx - period slide down
        else if eff_typ == 2 {
            let ch = &mut self.stm[chn];

            let mut tmp_eff = eff;
            if tmp_eff == 0 {
                tmp_eff = ch.porta_down_speed;
            }

            ch.porta_down_speed = tmp_eff;

            ch.real_period += tmp_eff as i16 * 4;
            if ch.real_period > 32000 - 1 {
                ch.real_period = 32000 - 1;
            }

            ch.out_period = ch.real_period as u16;
            ch.status   |= IS_PERIOD;
        }

        // 3xx - tone portamento
        else if eff_typ == 3 {
            self.tone_porta(chn)
        }

        // 4xy - vibrato
        else if eff_typ == 4 {
            self.vibrato(chn)
        }

        // 5xy - tone portamento + volume slide
        else if eff_typ == 5 {
            self.tone_porta(chn);
            self.volume(chn);
        }

        // 6xy - vibrato + volume slide
        else if eff_typ == 6 {
            self.vibrato2(chn);
            self.volume(chn);
        }

        // 7xy - tremolo
        else if eff_typ == 7 {
            let ch = &mut self.stm[chn];

            let tmp_eff = eff;
            if tmp_eff != 0 {
                if tmp_eff & 0x0F != 0 {
                    ch.trem_depth = tmp_eff & 0x0F;
                }
                if tmp_eff & 0xF0 != 0 {
                    ch.trem_speed = (tmp_eff >> 4) * 4;
                }
            }

            let mut tmp_trem = (ch.trem_pos / 4) & 0x1F;

            match (ch.wave_ctrl >> 4) & 3 {
                // 0: sine
                0 => tmp_trem = VIB_TAB[tmp_trem as usize],

                // 1: ramp
                1 => {
                    tmp_trem *= 8;
                    if ch.vib_pos >= 128 {
                        tmp_trem ^= 0xFF;  // FT2 bug, should've been TremPos
                    }
                },

                // 2/3: square
                _ => tmp_trem = 255,
            }

            tmp_trem = ((tmp_trem as u16 * ch.trem_depth as u16) / 64) as u8;

            let mut trem_vol: i16;
            if ch.trem_pos >= 128 {
                trem_vol = ch.real_vol as i16 - tmp_trem as i16;
                if trem_vol < 0 {
                    trem_vol = 0;
                }
            } else {
                trem_vol = ch.real_vol as i16 + tmp_trem as i16;
                if trem_vol > 64 {
                    trem_vol = 64;
                }
            }

            ch.out_vol = (trem_vol & 0x00FF) as i8;

            ch.trem_pos += ch.trem_speed;

            ch.status |= IS_VOL;
        }

        // Axy - volume slide
        else if eff_typ == 10 {
            self.volume(chn);  // actually volume slide
        }

        // Exy - E effects
        else if eff_typ == 14 {
            // E9x - note retrigger
            if eff & 0xF0 == 0x90 {
                if eff != 0x90 {  // E90 is handled in getNewNote()
                    if (self.song.tempo - self.song.timer) % (eff & 0x0F) as u16 == 0 {
                        self.start_tone(0, 0, 0, chn, &module);
                        self.retrig_envelope_vibrato(chn, &module);
                    }
                }
            }

            // ECx - note cut
            else if eff & 0xF0 == 0xC0 {
                let ch = &mut self.stm[chn];

                if ((self.song.tempo - self.song.timer) & 0x00FF) as u8 == eff & 0x0F {
                    ch.out_vol  = 0;
                    ch.real_vol = 0;
                    ch.status |= IS_VOL + IS_QUICKVOL;
                }
            }

            // EDx - note delay
            else if (eff & 0xF0) == 0xD0 {
                if ((self.song.tempo - self.song.timer) & 0x00FF) as u8 == eff & 0x0F {
                    let ton_typ = (self.stm[chn].ton_typ & 0x00FF) as u8;
                    self.start_tone(ton_typ, 0, 0, chn, &module);

                    if self.stm[chn].ton_typ & 0xFF00 != 0 {
                        self.retrig_volume(chn);
                    }

                    self.retrig_envelope_vibrato(chn, &module);

                    let ch = &mut self.stm[chn];

                    if ch.vol_kol_vol >= 0x10 && ch.vol_kol_vol <= 0x50 {
                        ch.out_vol  = (ch.vol_kol_vol - 16) as i8;
                        ch.real_vol = ch.out_vol;
                    } else if ch.vol_kol_vol >= 0xC0 && ch.vol_kol_vol <= 0xCF {
                        ch.out_pan = (ch.vol_kol_vol & 0x0F) << 4;
                    }
                }
            }
        }

        // Hxy - global volume slide
        else if eff_typ == 17 {
            {
                let ch = &mut self.stm[chn];

                let mut tmp_eff = eff;
                if tmp_eff != 0 {
                    tmp_eff = ch.glob_vol_slide_speed;
                }

                ch.glob_vol_slide_speed = tmp_eff;

                if tmp_eff & 0xF0 == 0 {
                    // unsigned clamp
                    if self.song.glob_vol >= tmp_eff  as u16 {
                        self.song.glob_vol -= tmp_eff as u16;
                    } else {
                        self.song.glob_vol = 0;
                    }
                } else {
                    // unsigned clamp
                    if self.song.glob_vol <= 64 - (tmp_eff >> 4) as u16 {
                        self.song.glob_vol += (tmp_eff >> 4) as u16;
                    } else {
                        self.song.glob_vol = 64;
                    }
                }
            }

            for i in 0..self.song.ant_chn as usize {
                self.stm[i].status |= IS_VOL;
            }
        }

        // Kxx - key off
        else if eff_typ == 20 {
            if (self.song.tempo - self.song.timer) & 31 == eff as u16 & 0x0F {
                self.key_off(chn, &module);
            }
        }

        // Pxy - panning slide
        else if eff_typ == 25 {
            let ch = &mut self.stm[chn];

            let mut tmp_eff = eff;
            if tmp_eff == 0 {
                tmp_eff = ch.panning_slide_speed;
            }

            ch.panning_slide_speed = tmp_eff;

            if tmp_eff & 0xF0 == 0 {
                // unsigned clamp
                if ch.out_pan >= tmp_eff {
                    ch.out_pan -= tmp_eff;
                } else {
                    ch.out_pan = 0;
                }
            } else {
                tmp_eff >>= 4;

                // unsigned clamp */
                if ch.out_pan <= 255 - tmp_eff {
                    ch.out_pan += tmp_eff;
                } else {
                    ch.out_pan = 255;
                }
            }

            ch.status |= IS_PAN;
        }

        // Rxy - multi note retrig
        else if eff_typ == 27 {
            self.multi_retrig(chn, &module);
        }

        // Txy - tremor
        else if eff_typ == 29 {
            let ch = &mut self.stm[chn];

            let mut tmp_eff = eff;
            if tmp_eff == 0 {
                tmp_eff = ch.tremor_save;
            }

            ch.tremor_save = tmp_eff;

            let mut tremor_sign = ch.tremor_pos & 0x80;
            let mut tremor_data = ch.tremor_pos & 0x7F;

            tremor_data.wrapping_sub(1);
            if tremor_data & 0x80 != 0 {
                if tremor_sign == 0x80 {
                    tremor_sign = 0x00;
                    tremor_data = tmp_eff & 0x0F;
                } else {
                    tremor_sign = 0x80;
                    tremor_data = tmp_eff >> 4;
                }
            }

            ch.tremor_pos = tremor_data | tremor_sign;

            ch.out_vol  = if tremor_sign != 0 { ch.real_vol } else { 0 };
            ch.status |= IS_VOL + IS_QUICKVOL;
        }
    }

/*
    static void voiceSetVolRamp(uint8_t chNr)
    {
        voice_t *v1, *v2;

        if (volumeRamping)
        {
            v1 = &voice[chNr]; // curr voice
            v2 = &voice[MAX_VOICES + chNr]; // ramp out voice

            if (v1->sampleData8 != NULL)
            {
                // copy current voice to ramp out voice
                memcpy(v2, v1, sizeof (voice_t));

                // set ramp out voice
                v2->faderDest  = 0.0f;
                v2->faderDelta = (v2->faderDest - v2->fader) * quickVolRampMul_f;
            }

            /* set ramp in for current voice */
            v1->fader      = 0.0f;
            v1->faderDest  = 1.0f;
            v1->faderDelta = (v1->faderDest - v1->fader) * quickVolRampMul_f;
        }
    }

    static void voiceUpdateVolumes(uint8_t i, uint8_t status)
    {
        float volL_f, volR_f, deltaMul_f;
        voice_t *v;

        v = &voice[i];

        volL_f = v->volume * v->panL;
        volR_f = v->volume * v->panR;

        if (volumeRamping)
        {
            if (!(status & IS_NyTon))
            {
                /* set vol ramp stuff */
                v->targetVolL = volL_f;
                v->targetVolR = volR_f;

                deltaMul_f = (status & IS_QuickVol) ? quickVolRampMul_f : tickVolRampMul_f;

                v->volDeltaL = (v->targetVolL - v->volumeL) * deltaMul_f;
                v->volDeltaR = (v->targetVolR - v->volumeR) * deltaMul_f;
            }
            else
            {
                v->targetVolL = v->volumeL = volL_f;
                v->targetVolR = v->volumeR = volR_f;

                v->volDeltaL = 0.0f;
                v->volDeltaR = 0.0f;
            }
        }
        else
        {
            v->volumeL = volL_f;
            v->volumeR = volR_f;
        }
    }
*/

    fn voice_set_source(&mut self, chn: usize, smp_num: u32, mut sample_length: u32, mut sample_loop_begin: u32,
        mut sample_loop_length: u32, mut sample_loop_end: u32, mut loop_flag: u8, sixteenbit: bool, stereo: bool,
        mut position: i32, mixer: &mut Mixer)
    {

/*
        voice_t *v;

        v = &voice[i];

        if ((sampleData == NULL) || (sampleLength < 1))
        {
            v->sampleData8  = NULL;
            v->sampleData16 = NULL;
            return;
        }
*/

        if position >= sample_length as i32 {
            position = 0;
            // reset voice?
            return;
        } else {
            mixer.set_sample(chn, smp_num as usize);
        }

        if sixteenbit {
            sample_loop_begin  = (sample_loop_begin  & 0xFFFFFFFE) / 2;
            sample_length      = (sample_length      & 0xFFFFFFFE) / 2;
            sample_loop_length = (sample_loop_length & 0xFFFFFFFE) / 2;
            sample_loop_end    = (sample_loop_end    & 0xFFFFFFFE) / 2;

            //v->sampleData16R = &v->sampleData16[sampleLength];
        }
/*
        else
        {
            v->sampleData8R = &v->sampleData8[sampleLength];
        }
*/

        if sample_loop_length < 2 {   // FT2 can do 1-sample loops, but we don't (for security reasons)
            loop_flag = 0;
        }

        mixer.set_loop_start(chn, sample_loop_begin);
        mixer.set_loop_end(chn, sample_loop_end);
        mixer.enable_loop(chn, loop_flag != 0);
        mixer.set_voicepos(chn, position as f64);
	
/*
        v->frac             = 0.0f;
        v->sample16bit      = sixteenbit ? true : false;
        v->loopingBackwards = false;
        v->samplePosition   = position;
        v->sampleLength     = sampleLength;
        v->sampleLoopBegin  = sampleLoopBegin;
        v->sampleLoopEnd    = sampleLoopEnd;
        v->sampleLoopLength = sampleLoopLength;;
        v->loop             = loopFlag ? ((loopFlag & 2) ? 2 : 1) : 0;
        v->stereo           = stereo ? true : false;
*/
    }

    fn voice_trigger(&mut self, chn: usize, module: &XmData, mut mixer: &mut Mixer) {
        let instr_nr = self.stm[chn].instr_nr;
        let smp_ptr = self.stm[chn].smp_ptr;

        // oxdz: sanity check
        if instr_nr == 0 {
            return
        }

        let ins = &module.instruments[instr_nr as usize - 1];

        // oxdz: sanity check
        if smp_ptr >= ins.samp.len() {
            return
        }

        let s = &ins.samp[smp_ptr];
        let smp_start_pos = self.stm[chn].smp_start_pos as i32;
        self.voice_set_source(chn, s.smp_num, s.len, s.rep_s, s.rep_l, s.rep_s + s.rep_l, s.typ & 3,
                              s.typ & 16 != 0, s.typ & 32 != 0, smp_start_pos, &mut mixer);
    }

    fn update_channel_vol_pan_frq(&mut self, module: &XmData, mut mixer: &mut Mixer) {
        for i in 0..MAX_VOICES {
            let status = self.stm[i].status;
            if status != 0 {
                self.stm[i].status = 0;

                /* this order is carefully selected, modification can result in unwanted behavior */
                //if status & IS_NYTON          != 0 { self.voiceSetVolRamp(i); }
                if status & IS_VOL            != 0 { mixer.set_volume(i, (self.stm[i].final_vol as usize) << 2); }   // 0..256 => 0..1024
                if status & IS_PAN            != 0 { mixer.set_pan(i, self.stm[i].final_pan as isize - 128); }
                //if status & (IS_VOL | IS_PAN) != 0 { self.voice_update_volumes(i, status); }
                if status & IS_PERIOD         != 0 { let frq = self.get_frequence_value(self.stm[i].final_period);
                                                     mixer.set_period(i, (1712 * 8363) as f64 / (frq * 4) as f64);
                                                   }
                if status & IS_NYTON          != 0 { self.voice_trigger(i, &module, &mut mixer); }
            }
        }
    }

    fn no_new_all_channels(&mut self, module: &XmData) {
        for i in 0..self.song.ant_chn as usize {
            self.do_effects(i, &module);
            self.fixa_envelope_vibrato(i, &module);
        }
    }

    fn get_next_pos(&mut self, module: &XmData) {
        if self.song.timer == 1 {
            self.song.patt_pos += 1;

            if self.song.patt_del_time != 0 {
                self.song.patt_del_time_2 = self.song.patt_del_time;
                self.song.patt_del_time = 0;
            }

            if self.song.patt_del_time_2 != 0 {
                self.song.patt_del_time_2 -= 1;
                if self.song.patt_del_time_2 != 0 {
                    self.song.patt_pos -= 1;
                }
            }

            if self.song.p_break_flag {
                self.song.p_break_flag = false;
                self.song.patt_pos = self.song.p_break_pos as i16;
            }

            if self.song.patt_pos >= self.song.patt_len || self.song.pos_jump_flag {
                self.song.patt_pos = self.song.p_break_pos as i16;
                self.song.p_break_pos = 0;
                self.song.pos_jump_flag = false;

                self.song.song_pos += 1;
                if self.song.song_pos >= self.song.len as i16 {
                      self.song.song_pos = self.song.rep_s as i16;
                }

                self.song.patt_nr = module.header.song_tab[self.song.song_pos as usize & 0xFF] as i16;
                self.song.patt_len = self.patt_lens[self.song.patt_nr as usize & 0xFF] as i16;
            }
        }
    }

    fn main_player(&mut self, module: &XmData) {  // periodically called from mixer
        /*if (musicPaused || !self.songPlaying)
        {
            for (i = 0; i < self.song.ant_chn; ++i)
                fixa_envelope_vibrato(&stm[i]);
        }
        else
        {*/
        let mut read_new_note = false;
        self.song.timer -= 1;
        if self.song.timer == 0 {
            self.song.timer = self.song.tempo;
            read_new_note = true;
        }

        if read_new_note {
            if self.song.patt_del_time_2 == 0 {
                for i in 0..self.song.ant_chn as usize {
                    //if patt[self.song.patt_nr] != NULL {
                    let pat_num = self.song.patt_nr as usize;
                    let pat_pos = self.song.patt_pos;
                    self.get_new_note(i, module.patterns[pat_num].event(pat_pos, i), &module);
                    //} else {
                    //    get_new_note(i, &nilPatternLine[(self.song.patt_pos * MAX_VOICES) + i]);
                    //}

                    self.fixa_envelope_vibrato(i, &module);
                }
            } else {
                self.no_new_all_channels(&module);
            }
        } else {
            self.no_new_all_channels(&module);
        }

        self.get_next_pos(&module);
        //}
    }

    fn set_pos(&mut self, song_pos: i16, patt_pos: i16, module: &XmData) {
        debug!("set_pos pat={} row={}", song_pos, patt_pos);
        if song_pos > -1 {
            self.song.song_pos = song_pos;
            if self.song.len > 0 && self.song.song_pos >= self.song.len as i16 {
                self.song.song_pos = self.song.len as i16 - 1;
            }

            self.song.patt_nr = module.header.song_tab[song_pos as usize] as i16;
            let patt_nr = self.song.patt_nr as usize;
            self.song.patt_len = self.patt_lens[patt_nr] as i16;
        }

        if patt_pos > -1 {
            self.song.patt_pos = patt_pos;
            if self.song.patt_pos >= self.song.patt_len {
                self.song.patt_pos  = self.song.patt_len - 1;
            }
        }

        self.song.timer = 1;
    }

}



impl FormatPlayer for Ft2Play {
    fn start(&mut self, data: &mut PlayerData, mdata: &ModuleData, _mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<XmData>().unwrap();

        let h = &module.header;

        for p in &module.patterns {
            self.patt_lens.push(p.patt_len);
        }

        self.song.glob_vol = 64;
        self.song.len = h.len;
        self.song.rep_s = h.rep_s;
        self.song.ant_chn = h.ant_chn as u8;
        self.song.speed = if h.def_speed != 0 { h.def_speed } else { 125 };
        self.song.tempo = if h.def_tempo != 0 { h.def_tempo } else { 6 };
        self.song.ant_instrs = h.ant_instrs;
        self.song.ant_ptn = h.ant_ptn;
        self.song.ver = h.ver;
        self.linear_frq_tab = h.flags & 1 != 0;


        // generate tables

        // generate log table (value-exact to FT2's table)
        for i in 0..(4 * 12 * 16) {
            self.log_tab.push((((256.0 * 8363.0) * ((i as f64 / 768.0) * 2.0_f64.ln()).exp()) + 0.5) as u32);
        }

        if self.linear_frq_tab {
            // generate linear table (value-exact to FT2's table)
            for i in 0..((12 * 10 * 16) + 16) {
                self.note2period.push((((12 * 10 * 16) + 16) * 4) - (i * 4));
            }
        } else {
            // generate amiga period table (value-exact to FT2's table, except for last 17 entries)
            for i in 0..10 {
                for j in 0..(if i == 9 {96 + 8} else {96}) {
                    let note_val = (((AMIGA_FINE_PERIOD[j % 96] * 64) + ((1 << i) - 1)) >> (i + 1)) as i16;
                    /* NON-FT2: j % 96. added for safety. we're patching the values later anyways. */

                    self.note2period.push(note_val);
                    self.note2period.push(note_val);
                }
            }

            /* interpolate between points (end-result is exact to FT2's table, except for last 17 entries) */
            for i in 0..(12 * 10 * 8) + 7 {
                self.note2period[(i * 2) + 1] = ((self.note2period[i * 2] as i32 + self.note2period[(i * 2) + 2] as i32) / 2) as i16;
            }

            // the amiga linear period table has its 17 last entries generated wrongly.
            // the content seem to be garbage because of an 'out of boundaries' read from AmigaFinePeriods.
            // these 17 values were taken from a memdump of FT2 in DOSBox.
            // they might change depending on what you ran before FT2, but let's not make it too complicated.

            /*amigaPeriods[1919] = 22; amigaPeriods[1920] = 16; amigaPeriods[1921] =  8; amigaPeriods[1922] =  0;
            amigaPeriods[1923] = 16; amigaPeriods[1924] = 32; amigaPeriods[1925] = 24; amigaPeriods[1926] = 16;
            amigaPeriods[1927] =  8; amigaPeriods[1928] =  0; amigaPeriods[1929] = 16; amigaPeriods[1930] = 32;
            amigaPeriods[1931] = 24; amigaPeriods[1932] = 16; amigaPeriods[1933] =  8; amigaPeriods[1934] =  0;
            amigaPeriods[1935] =  0;*/
        }

        // generate auto-vibrato table (value-exact to FT2's table)
        for i in 0..256 {
            self.vib_sine_tab.push((((64.0 * ((-i as f64 * (2.0 * PI)) / 256.0).sin()) + 0.5).floor()) as i8);
        }

        self.set_pos(0, 0, &module);

        data.speed = self.song.tempo as usize;
        data.tempo = self.song.speed as f32;

        data.initial_speed = data.speed;
        data.initial_tempo = data.tempo;
    }

    fn play(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<XmData>().unwrap();

        self.main_player(&module);
        self.update_channel_vol_pan_frq(&module, &mut mixer);

        data.frame = ((self.song.tempo - self.song.timer + 1) % self.song.tempo) as usize;
        data.row = self.song.patt_pos as usize;
        data.pos = self.song.song_pos as usize;
        data.speed = self.song.tempo as usize;
        data.tempo = self.song.speed as f32;
        data.time += 20.0 * 125.0 / data.tempo as f32;

        /*if self.position_jump_cmd {
            data.pos = self.ft_song_pos.wrapping_add(1) as usize;
            self.position_jump_cmd = false;
        }

        data.inside_loop = false;
        for chn in 0..8 {
            data.inside_loop |= self.ft_chantemp[chn].inside_loop;
        }*/

    }

    fn reset(&mut self) {
    }

    unsafe fn save_state(&self) -> State {
        self.save()
    }

    unsafe fn restore_state(&mut self, state: &State) {
        self.restore(&state);
    }
}

