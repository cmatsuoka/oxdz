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
    p_break_flag   : u8,
    p_break_pos    : u8,
    pos_jump_flag  : u8,
    //song_tab       : [u8; 256],
    ver            : u16,
    name           : String,
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
    porta_dir             : bool,
    gliss_funk            : u8,
    vib_pos               : u8,
    trem_po               : u8,
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

    //sampleTyp *smpPtr;
    //instrTyp *instrPtr;
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
pub struct Ft2Play {
    speed_val           : u32,
    real_replay_rate    : u32,
    f_audio_freq        : f32,
    quick_vol_ramp_mul_f: f32,
    tick_vol_ramp_mul_f : f32,

    stm                 : [StmTyp; MAX_VOICES],
} 

impl Ft2Play {
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
        let mut ch = &mut self.stm[chn];
        ch.real_vol = ch.old_vol;
        ch.out_vol  = ch.old_vol;
        ch.out_pan  = ch.old_pan;
        ch.status  |= (IS_VOL + IS_PAN + IS_QUICKVOL);
    }
}


impl FormatPlayer for Ft2Play {
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

