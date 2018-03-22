extern crate memmap;
extern crate oxdz;
extern crate sdl2;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Write, stdout};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use memmap::Mmap;
use oxdz::{Oxdz, FrameInfo};
use oxdz::player::Player;
use sdl2::audio::{AudioCallback, AudioSpecDesired};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("usage: {} <filename>", Path::new(&args[0]).file_name().unwrap().to_str().unwrap());
        return;
    }

    match run(args) {
        Ok(_)  => {},
        Err(e) => eprintln!("error: {}", e),
    }
}

struct ModPlayer<'a> {
    player: Player<'a>,
    data: Arc<Mutex<FrameInfo>>,
}

impl<'a> AudioCallback for ModPlayer<'a> {
    type Channel = i16;

    fn callback(&mut self, mut out: &mut [i16]) {
        {
            let mut fi = self.data.lock().unwrap();
            self.player.info(&mut fi);
        }
        self.player.fill_buffer(&mut out, 0);
    }
}

fn run(args: Vec<String>) -> Result<(), Box<Error>> {

    let filename = &args[1];
    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };

    let oxdz = Oxdz::new(&mmap[..], 44100, "")?;

    // Display basic module information
    println!("Title : {}", oxdz.module.title());
    println!("Format: {}", oxdz.module.creator);

    let mut player = oxdz.player().unwrap();
    player.start();

    // From Rust-SDL2 SquareWave example
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),  // stereo
        samples: None,      // default buffer size
    };

    let data = Arc::new(Mutex::new(FrameInfo::new()));

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // Show obtained AudioSpec
        println!("{:?}", spec);

        // initialize the audio callback
        ModPlayer { player, data: data.clone() }
    }).unwrap();

    // Start playback
    device.resume();

    loop {
        {
            let fi = data.lock().unwrap();
            print!("pos:{:3} - row:{:3} \r", fi.pos, fi.row);
            let _ = stdout().flush();
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
