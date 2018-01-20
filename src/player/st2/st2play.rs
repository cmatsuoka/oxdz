use module::Module;
use player::{PlayerData, Virtual, FormatPlayer};
use format::stm::{StmPatterns, StmInstrument};

/// ST2Play Scream Tracker 2 replayer
///
/// An oxdz player based on st2play written by Sergei "x0r" Kolzun

pub struct St2Play {
    state: Vec<ST2Channel>,
}

impl St2Play {
    pub fn new(module: &Module) -> Self {
        St2Play {
            state: vec![ST2Channel::new(); module.chn],
        }
    }
}

const LFO_TABLE: &'static[i16] = &[
       0,   24,   49,   74,   97,  120,  141,  161,  180,  197,  212,  224,  235,  244,  250,  253,
     255,  253,  250,  244,  235,  224,  212,  197,  180,  161,  141,  120,   97,   74,   49,   24,
       0,  -24,  -49,  -74,  -97, -120, -141, -161, -180, -197, -212, -224, -235, -244, -250, -253,
    -255, -253, -250, -244, -235, -224, -212, -197, -180, -161, -141, -120,  -97,  -74,  -49,  -24,
       0
];


impl FormatPlayer for St2Play {
    fn play(&mut self, data: &mut PlayerData, module: &Module, mut virt: &mut Virtual) {
    }

    fn reset(&mut self) {
    }
}

#[derive(Default,Clone)]
struct ST2Channel {
    on               : bool,
    empty            : bool,
    row              : u16,
    pattern_data_offs: usize,
    event_note       : u16,
    event_volume     : u8,
    event_smp        : u16,
    event_cmd        : u16,
    event_infobyte   : u16,
    last_note        : u16,
    period_current   : u16,
    period_target    : u16,
    vibrato_current  : u16,
    tremor_counter   : u16,
    tremor_state     : u16,
    //uint8_t *smp_name;
    //uint8_t *smp_data_ptr;
    //uint16_t smp_loop_end;
    //uint16_t smp_loop_start;
    //uint16_t smp_c2spd;
    //uint32_t smp_position;
    //uint32_t smp_step;
    //uint16_t volume_initial;
    volume_current  : u16,
    //uint16_t volume_meter;
    //uint16_t volume_mix;
}

impl ST2Channel {

    pub fn new() -> Self {
        Default::default()
    }
}

