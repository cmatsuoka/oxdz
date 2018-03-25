extern crate byteorder;

#[macro_use]
extern crate save_restore_derive;

#[macro_use]
mod util;

mod player;
mod format;
mod mixer;

pub mod module;
pub use player::FrameInfo;
pub use player::PlayerInfo;
pub use module::Module;

use std::error;
use std::fmt;
use std::io;

pub const PERIOD_BASE  : f64 = 13696.0;  // C0 period
pub const MAX_RATE     : i32 = 96000;
pub const MIN_RATE     : i32 = 4000;
pub const MIN_BPM      : i32 = 20;
// frame rate = (50 * bpm / 125) Hz
// frame size = (sampling rate * channels) / frame rate
pub const MAX_FRAMESIZE: usize = (5 * MAX_RATE / MIN_BPM) as usize;
pub const MAX_KEYS     : usize = 128;
pub const MAX_CHANNELS : usize = 64;
pub const MAX_SEQUENCES: usize = 16;

#[derive(Default)]
pub struct ModuleInfo {
    pub title      : String,            // module title
    pub format_id  : &'static str,      // format identifier
    pub description: String,            // format description
    pub creator    : String,            // tracker name
    pub channels   : usize,             // number of mixer channels
    pub player     : &'static str,      // primary player for this format
    pub total_time : u32,               // replay time in ms
}

impl ModuleInfo {
    pub fn new() -> Self {
        Default::default()
    }
}

pub struct Oxdz<'a> {
    pub player   : player::Player<'a>,
    pub rate     : u32,
    pub player_id: String,
}

impl<'a> Oxdz<'a> {
    pub fn new(b: &[u8], rate: u32, player_id: &str) -> Result<Self, Error> {
        let mut module = format::load(&b, &player_id)?;
        let id = (if player_id.is_empty() { module.player } else { player_id }).to_owned();

        // import the module if needed
        module = player::list_by_id(&id)?.import(module)?;

        let mut player = player::Player::find(module, rate, &id, "")?;
        player.scan();
        player.start();

        Ok(Oxdz {
            player,
            rate,
            player_id: id,
        })
    }

    pub fn module(&'a self) -> &'a module::Module {
        &self.player.module
    }

    pub fn player_info(&self) -> Result<player::PlayerInfo, Error> {
        Ok(player::list_by_id(&self.player_id)?.info())
    }

    pub fn module_info(&self, mi: &mut ModuleInfo) {
        mi.title = self.player.module.title().to_owned();
        mi.format_id = self.player.module.format_id;
        mi.description = self.player.module.description.to_owned();
        mi.creator = self.player.module.creator.to_owned();
        mi.channels = self.player.module.channels;
        mi.player = self.player.module.player;
        mi.total_time = self.player.total_time;
    }

    pub fn frame_info(&mut self, mut fi: &mut FrameInfo) -> &mut Self {
        self.player.info(&mut fi);
        self
    }

    pub fn fill_buffer(&mut self, mut buffer: &mut [i16], loops: usize) -> &mut Self {
        self.player.fill_buffer(&mut buffer, loops);
        self
    }

    pub fn play_frame(&mut self) -> &mut Self {
        self.player.play_frame();
        self
    }

    pub fn buffer(&self) -> &[i16] {
        self.player.buffer()
    }

    pub fn set_mute(&mut self, chn: usize, val: bool) -> &mut Self {
        self.player.set_mute(chn, val);
        self
    }

    pub fn set_mute_all(&mut self, val: bool) -> &mut Self {
        self.player.set_mute_all(val);
        self
    }

    pub fn set_position(&mut self, pos: usize) -> &mut Self {
        self.player.set_position(pos);
        self
    }

    pub fn set_interpolator(&mut self, name: &str) -> Result<&mut Self, Error> {
        self.player.set_interpolator(name)?;
        Ok(self)
    }

/*
    pub fn player(&'a mut self) -> &'a mut player::Player {
        &mut self.player
    }
*/
}

pub fn player_list() -> Vec<PlayerInfo> {
    player::list()
}

pub fn format_list() -> Vec<Box<format::Loader>> {
    format::list()
}

#[derive(Debug)]
pub enum Error {
    Format(String),
    Player(String),
    Load(String),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Format(ref descr) => write!(f, "{}", descr),
            &Error::Player(ref descr) => write!(f, "{}", descr),
            &Error::Load(ref descr)   => write!(f, "{}", descr),
            &Error::Io(ref err)       => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Format(_)   => "Unsupported module format",
            Error::Player(_)   => "Can't play module",
            Error::Load(_)     => "Can't load module data",
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            _                  => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

