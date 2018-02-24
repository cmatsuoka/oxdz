extern crate byteorder;

#[macro_use]
mod util;

pub mod format;
pub mod mixer;
pub mod module;
pub mod player;
pub use player::FrameInfo;

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

pub struct Oxdz {
    pub module   : module::Module,
    pub player_id: String,
}

impl<'a> Oxdz {
    pub fn new(b: &[u8], mut player_id: &str) -> Result<Self, Error> {
        let mut module = format::load(&b, &player_id)?;
        let player = module.player.to_owned();
        let id = (if player_id.is_empty() { module.player } else { player_id }).to_owned();

        // import the module if needed
        module = player::list_by_id(&id)?.import(module)?;

        Ok(Oxdz {
            module,
            player_id: id,
        })
    }

    pub fn module(&'a self) -> &'a module::Module {
        &self.module
    }

    pub fn player_info(&self) -> Result<player::PlayerInfo, Error> {
        Ok(player::list_by_id(&self.player_id)?.info())
    }

    pub fn player(&mut self) -> Result<player::Player, Error> {
        let player = player::Player::find(&mut self.module, &self.player_id, "")?;
        Ok(player)
    }
}


#[derive(Debug)]
pub enum Error {
    Format(String),
    Load(String),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::Format(ref descr) => write!(f, "{}", descr),
            &Error::Load(ref descr)   => write!(f, "{}", descr),
            &Error::Io(ref err)       => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Format(_)   => "Unsupported module format",
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

