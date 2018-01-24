mod virt;
mod scan;
mod protracker;
mod st2;
mod st3;

pub use player::virt::Virtual;
pub use mixer::Mixer;

use std::cmp;
use module::{Module, ModuleData};
use util::MemOpExt;
use ::*;

// For the player list

pub struct PlayerInfo {
    pub id         : &'static str,
    pub name       : &'static str,
    pub description: &'static str,
    pub author     : &'static str,
    pub accepts    : &'static [&'static str],
}

pub trait PlayerListEntry {
    fn info(&self) -> PlayerInfo;
    fn player(&self, module: &Module) -> Box<FormatPlayer>;
}


// Trait for format-specific players

pub trait FormatPlayer: Send + Sync {
    fn start(&mut self, &mut PlayerData, &ModuleData);
    fn play(&mut self, &mut PlayerData, &ModuleData, &mut Virtual);
    fn reset(&mut self);
}

#[derive(Default)]
pub struct PlayerData {
    pub pos  : usize,
    pub row  : usize,
    pub frame: usize,
    pub song : usize,
    pub speed: usize,
    pub tempo: usize,

    initial_speed: usize,
    initial_tempo: usize,
}

impl PlayerData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.pos   = 0;
        self.row   = 0;
        self.frame = 0;
        self.song  = 0;
        self.speed = self.initial_speed;
        self.tempo = self.initial_tempo;
    }
}


pub struct Player<'a> {
    pub data     : PlayerData,
    module       : &'a Module<'a>,
    format_player: Box<FormatPlayer>,
    virt         : Virtual<'a>,
    loop_count   : usize,
    end          : bool,

    // for buffer fill
    consumed     : usize,
    in_pos       : usize,
    in_size      : usize,
    
}

impl<'a> Player<'a> {
    pub fn find_player(module: &'a Module, player_id: &str) -> Result<Self, Error> {

        let format_player = Player::find_by_id(player_id)?.player(&module);

        let virt = Virtual::new(module.data.channels(), &module.data.samples(), false);
        Ok(Player {
            data      : PlayerData::new(),
            module,
            format_player,
            virt,
            loop_count: 0,
            end       : false,
            consumed  : 0,
            in_pos    : 0,
            in_size   : 0,
        })
    }

    pub fn list() -> Vec<Box<PlayerListEntry>> {
        vec![
            Box::new(st3::St3),
            Box::new(st2::St2),
            Box::new(protracker::Pt21a),
        ]
    }

    fn find_by_id(player_id: &str) -> Result<Box<PlayerListEntry>, Error> {
        for p in Self::list() {
            if player_id == p.info().id {
                return Ok(p)
            }
        }
        Err(Error::Format("player not found"))
    }

    pub fn scan(&mut self) -> &Self {
        self.data.reset();
        self
    }

/*
    pub fn restart(&mut self) -> &Self {
        self.data.pos = 0;
        self.data.row = 0;
        self.data.song = 0;
        self.data.frame = 0;
        self.data.speed = 6;
        self.format_player.reset();
        self
    }
*/

    pub fn start(&mut self) -> &mut Self {
        self.format_player.start(&mut self.data, &*self.module.data);
        self
    }

    pub fn play_frame(&mut self) -> &mut Self {
        self.format_player.play(&mut self.data, &*self.module.data, &mut self.virt);
        self.virt.set_tempo(self.data.tempo);
        self.virt.mix();
        self
    }

    pub fn fill_buffer(&mut self, out_buffer: &mut [i16], loops: usize) {
        let mut filled = 0;
        let size = out_buffer.len();

        // Fill buffer
        while filled < size {
            // Check if buffer full
            if self.consumed == self.in_size {
                self.play_frame();

                // Check end of module
                if self.end() || (loops > 0 && self.loop_count >= loops) {
                    // Start of frame, return end of replay
                    if filled == 0 {
                        self.consumed = 0;
                        self.in_size = 0;
                        return;
                    } else {
                        self.end = false;
                    }

                    // Clear rest of the buffer
                    out_buffer[filled..].fill(0, size - filled);
                }

                self.consumed = 0;
                self.in_pos = 0;
                self.in_size = self.buffer().len();
            }

            // Copy frame data to user buffer
            let copy_size = cmp::min(size - filled, self.in_size - self.consumed);
            out_buffer[filled..filled+copy_size].copy_from_slice(&self.buffer()[self.consumed..self.consumed+copy_size]);
            self.consumed += copy_size;
            filled += copy_size;
        }
    }

    pub fn end(&self) -> bool {
        self.end
    }

    pub fn info(&mut self, info: &mut FrameInfo) -> &mut Self {
        info.pos = self.data.pos;
        info.row = self.data.row;
        info.song = self.data.song;
        info.frame = self.data.frame;
        info.speed = self.data.speed;
        info.tempo = self.data.tempo;
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
    pub pos  : usize,
    pub row  : usize,
    pub frame: usize,
    pub song : usize,
    pub tempo: usize,
    pub speed: usize,
}

impl FrameInfo {
    pub fn new() -> Self {
        Default::default()
    }
}
