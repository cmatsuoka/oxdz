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
}

impl PlayerData {
    pub fn new(module: &Module) -> Self {
        PlayerData {
            pos  : 0,
            row  : 0,
            frame: 0,
            song : 0,
            speed: module.speed,
        }
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
        let mixer = Mixer::new(module.chn, &module.sample);
        let virt = Virtual::new(mixer, module.chn, false);
        Player {
            data : PlayerData::new(&module),
            module,
            format_player, //: &mut *format_player,
            virt,
        }
    }

    pub fn restart(&mut self) -> &Self {
        self.data.pos = 0;
        self.data.row = 0;
        self.data.song = 0;
        self.data.frame = 0;
        self.data.speed = self.module.speed;
        self
    }

    pub fn play_frame(&mut self) -> &Self {
        self.format_player.play(&mut self.data, &self.module, &mut self.virt);
        self.next_frame();
        self
    }

    fn next_frame(&mut self) {
        self.data.frame += 1;
        if self.data.frame >= self.data.speed {
            self.next_row();
        }
    }

    fn next_row(&mut self) {
        self.data.frame = 0;
        self.data.row += 1;
        if self.data.row > self.module.patterns.len(self.module.orders.pattern(&self.data)) {
            self.next_pattern();
        }
    }

    fn next_pattern(&mut self) {
        self.data.row = 0;  // FIXME: pattern break row
        self.data.pos += 1;
        if self.data.pos > self.module.orders.num(self.data.song) {
            // FIXME: add loop control
            self.restart();
        }
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
}
