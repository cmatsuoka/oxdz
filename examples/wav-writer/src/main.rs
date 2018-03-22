extern crate memmap;
extern crate oxdz;
extern crate riff_wave;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use memmap::Mmap;
use oxdz::{Oxdz, FrameInfo};
use riff_wave::WaveWriter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("usage: {} <filename> <secs>", Path::new(&args[0]).file_name().unwrap().to_str().unwrap());
        return;
    }

    match run(args) {
        Ok(_)  => {},
        Err(e) => eprintln!("error: {}", e),
    }
}


fn run(args: Vec<String>) -> Result<(), Box<Error>> {

    let filename = &args[1];
    let replay_time = parse_num(&args[2])? as f32 * 1000.0;

    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };

    let mut oxdz = Oxdz::new(&mmap[..], 44100, "")?;

    // Display basic module information
    println!("Title : {}", oxdz.module.title());
    println!("Format: {}", oxdz.module.creator);

    let mut player = oxdz.player()?;
    let mut fi = FrameInfo::new();

    // Prepare to write a wav file
    let out_filename = "out.wav";
    let file = File::create(out_filename)?;
    let writer = BufWriter::new(file);
    let mut wave_writer = try!(WaveWriter::new(2, 44100, 16, writer));

    player.start();

    let mut frames = 0;
    loop {
        let buffer = player.info(&mut fi).play_frame().buffer();
	if fi.loop_count > 0 || fi.time > replay_time {
            break
        }
        for s in buffer {
            wave_writer.write_sample_i16(*s)?;
        }
        frames += 1;
    }

    println!("wrote {}: {} frames ({:.1}s)", out_filename, frames, fi.time / 1000.0);

    Ok(())
}

fn parse_num(s: &str) -> Result<usize, std::num::ParseIntError> {
    if s.starts_with("0x") {
        usize::from_str_radix(&s[2..], 16)
    } else {
        s.parse()
    }
}
