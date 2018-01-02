use module::Module;
use ::*;

pub struct Player<'a> {
    pos   : usize,
    row   : usize,
    song  : usize,
    module: &'a Module,
}

impl<'a> Player<'a> {
    pub fn with_module(module: &'a Module) -> Self {
        Player {
            pos : 0,
            row : 0,
            song: 0,
            module,
        }
    }

    pub fn reset(&mut self) -> &Self {
        self.pos = 0;
        self.row = 0;
        self.song = 0;
        self
    }

    pub fn play_frame(&mut self) -> &Self {
        for chn in 0..self.module.chn {

        }
        self
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn song(&self) -> usize {
        self.song
    }

    pub fn set_position(&mut self, pos: usize) -> &Self {
        self.pos = pos;
        self
    }

    pub fn set_song(&mut self, song: usize) -> &Self {
        self.song = song;
        self.pos = 0;
        self
    }
}
