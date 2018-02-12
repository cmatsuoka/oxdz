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

#[derive(Debug)]
pub enum PeriodType {
    Linear,
    Amiga,
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

