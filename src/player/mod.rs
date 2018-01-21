mod virt;
mod scan;
mod protracker;
mod st2;

pub use player::virt::Virtual;
pub use mixer::Mixer;

use module::Module;
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

pub trait FormatPlayer {
    fn start(&mut self, &mut PlayerData, &Module);
    fn play(&mut self, &mut PlayerData, &Module, &mut Virtual);
    fn reset(&mut self);
}

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
    pub fn new(module: &Module) -> Self {
        PlayerData {
            pos  : 0,
            row  : 0,
            frame: 0,
            song : 0,
            speed: module.speed,
            tempo: module.tempo,

            initial_speed: module.speed,
            initial_tempo: module.tempo,
        }
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
    pub data  : PlayerData,
    module: &'a Module,
    format_player: Box<FormatPlayer>,
    virt  : Virtual<'a>,
}

impl<'a> Player<'a> {
    pub fn find_player(module: &'a Module, player_id: &str) -> Result<Self, Error> {

        let format_player = Player::find_by_id(player_id)?.player(&module);

        let virt = Virtual::new(module.chn, &module.sample, false);
        Ok(Player {
            data : PlayerData::new(&module),
            module,
            format_player,
            virt,
        })
    }

    pub fn list() -> Vec<Box<PlayerListEntry>> {
        vec![
            Box::new(protracker::Pt21a),
            Box::new(st2::St2),
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

    pub fn restart(&mut self) -> &Self {
        self.data.pos = 0;
        self.data.row = 0;
        self.data.song = 0;
        self.data.frame = 0;
        self.data.speed = self.module.speed;
        self.format_player.reset();
        self
    }

    pub fn start(&mut self) -> &mut Self {
        self.format_player.start(&mut self.data, &self.module);
        self
    }

    pub fn play_frame(&mut self) -> &mut Self {
        self.format_player.play(&mut self.data, &self.module, &mut self.virt);
        self.virt.mix(self.data.tempo);
        self
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
