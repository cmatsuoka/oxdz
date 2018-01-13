mod virt;

pub use player::virt::Virtual;
pub use mixer::Mixer;
use format::FormatPlayer;
use module::Module;


pub struct PlayerData {
    pub pos  : usize,
    pub row  : usize,
    pub frame: usize,
    pub song : usize,
    pub speed: usize,
    pub bpm  : usize,

    initial_speed: usize,
    initial_bpm  : usize,
}

impl PlayerData {
    pub fn new(module: &Module) -> Self {
        PlayerData {
            pos  : 0,
            row  : 0,
            frame: 0,
            song : 0,
            speed: module.speed,
            bpm  : module.bpm,

            initial_speed: module.speed,
            initial_bpm  : module.bpm,
        }
    }

    pub fn reset(&mut self) {
        self.pos   = 0;
        self.row   = 0;
        self.frame = 0;
        self.song  = 0;
        self.speed = self.initial_speed;
        self.bpm   = self.initial_bpm;
    }
}

pub struct Player<'a> {
    pub data  : PlayerData,
    module: &'a Module,
    format_player: Box<FormatPlayer>,
    virt  : Virtual<'a>,
}

impl<'a> Player<'a> {
    pub fn new(module: &'a Module, format_player: Box<FormatPlayer>) -> Self {
        let virt = Virtual::new(module.chn, &module.sample, false);
        Player {
            data : PlayerData::new(&module),
            module,
            format_player, //: &mut *format_player,
            virt,
        }
    }

    pub fn scan(&mut self) -> &Self {
        self.data.reset();
        self
    }

    pub fn restart(&mut self) -> &Self {
        self.data.pos = 0;
        self.data.row = 0;
        self.data.song = 0;
        self.data.frame = 0;
        self.data.speed = self.module.speed;
        self.format_player.reset();
        self
    }

    pub fn play_frame(&mut self) -> &Self {
        self.format_player.play(&mut self.data, &self.module, &mut self.virt);
        self.virt.mix(self.data.bpm);
        self
    }

    pub fn info(&self, info: &mut FrameInfo) -> &Self {
        info.pos = self.data.pos;
        info.row = self.data.row;
        info.song = self.data.song;
        info.frame = self.data.frame;
        info.speed = self.data.speed;
        info.bpm = self.data.bpm;
        self
    }

    pub fn position(&self) -> usize {
        self.data.pos
    }

    pub fn row(&self) -> usize {
        self.data.row
    }

    pub fn frame(&self) -> usize {
        self.data.frame
    }

    pub fn song(&self) -> usize {
        self.data.song
    }

    pub fn set_position(&mut self, pos: usize) -> &Self {
        self.data.pos = pos;
        self.set_row(0)
    }

    pub fn set_row(&mut self, row: usize) -> &Self {
        self.data.row = row;
        self.set_frame(0)
    }

    pub fn set_frame(&mut self, frame: usize) -> &Self {
        self.data.frame = frame;
        self
    }

    pub fn set_song(&mut self, song: usize) -> &Self {
        self.data.song = song;
        //self.data.pos = 0; FIXME: songs may start at pos != 0
        self
    }

    pub fn buffer(&self) -> &[i16] {
        self.virt.buffer()
    }
}


#[derive(Default)]
pub struct FrameInfo {
    pub pos: usize,
    pub row: usize,
    pub frame: usize,
    pub song: usize,
    pub bpm: usize,
    pub speed: usize,
}

impl FrameInfo {
    pub fn new() -> Self {
        Default::default()
    }
}
