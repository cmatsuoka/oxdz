mod virt;

pub use player::virt::Virtual;
pub use mixer::Mixer;
use module::Module;
use ::*;



pub struct Player<'a> {
    pos   : usize,
    row   : usize,
    frame : usize,
    song  : usize,
    speed : usize,
    module: &'a Module,

    virt  : Virtual<'a>,
}

impl<'a> Player<'a> {
    pub fn new(module: &'a Module) -> Self {
        let mixer = Mixer::new(module.chn, &module.sample);
        let virt = Virtual::new(mixer, module.chn, false);

        Player {
            pos  : 0,
            row  : 0,
            frame: 0,
            song : 0,
            speed: module.speed,
            module,

            virt,
        }
    }

    pub fn restart(&mut self) -> &Self {
        self.pos = 0;
        self.row = 0;
        self.song = 0;
        self.frame = 0;
        self.speed = self.module.speed;
        self
    }

    pub fn play_frame(&mut self) -> &Self {
        self.module.player.play(&self, &self.module);
        self.next_frame();
        self
    }


    fn next_frame(&mut self) {
        self.frame += 1;
        if self.frame >= self.speed {
            self.next_row();
        }
    }

    fn next_row(&mut self) {
        self.frame = 0;
        self.row += 1;
        if self.row > self.module.patterns.len(self.module.orders.pattern(&self)) {
            self.next_pattern();
        }
    }

    fn next_pattern(&mut self) {
        self.row = 0;  // FIXME: pattern break row
        self.pos += 1;
        if self.pos > self.module.orders.num(self.song) {
            // FIXME: add loop control
            self.restart();
        }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn frame(&self) -> usize {
        self.frame
    }

    pub fn song(&self) -> usize {
        self.song
    }

    pub fn set_position(&mut self, pos: usize) -> &Self {
        self.pos = pos;
        self.set_row(0)
    }

    pub fn set_row(&mut self, row: usize) -> &Self {
        self.row = row;
        self.set_frame(0)
    }

    pub fn set_frame(&mut self, frame: usize) -> &Self {
        self.frame = frame;
        self
    }

    pub fn set_song(&mut self, song: usize) -> &Self {
        self.song = song;
        //self.pos = 0; FIXME: songs may start at pos != 0
        self
    }
}
