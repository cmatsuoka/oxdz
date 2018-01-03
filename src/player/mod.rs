use module::Module;
use ::*;

pub struct Player<'a> {
    pos   : usize,
    row   : usize,
    frame : usize,
    song  : usize,
    module: &'a Module,
}

impl<'a> Player<'a> {
    pub fn with_module(module: &'a Module) -> Self {
        Player {
            pos  : 0,
            row  : 0,
            frame: 0,
            song : 0,
            module,
        }
    }

    pub fn reset(&mut self) -> &Self {
        self.pos = 0;
        self.row = 0;
        self.song = 0;
        self.frame = 0;
        self
    }

    pub fn play_frame(&mut self, frame: usize) -> &Self {
        for chn in 0..self.module.chn {
            self.module.playframe.play(&self, &self.module)
        }
        self
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
