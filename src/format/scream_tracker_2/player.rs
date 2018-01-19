use module::Module;
use format::FormatPlayer;
use player::{PlayerData, Virtual};
use format::scream_tracker_2::{StmPatterns, StmInstrument};

/// ST2Play Scream Tracker 2 replayer
///
/// An oxdz player based on st2play written by Sergei "x0r" Kolzun

pub struct StmPlayer {
    name : &'static str,
    state: Vec<ST2Channel>,
}

impl StmPlayer {
    pub fn new(module: &Module) -> Self {
        StmPlayer {
            name : r#""Vinterstigen" 0.1 PT2.1A replayer"#,
            state: vec![ST2Channel::new(); module.chn],

        }
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
