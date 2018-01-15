extern crate memmap;
extern crate oxdz;
extern crate riff_wave;
extern crate getopts;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use getopts::{Matches, Options};
use memmap::Mmap;
use oxdz::{format, module, player, FrameInfo};
use riff_wave::WaveWriter;

fn main() {

    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optflag("h", "help", "display usage information and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    if matches.opt_present("h") ||  matches.free.len() < 1 {
        let brief = format!("Usage: {} [options] filename", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    match run(&matches.free[0]) {
        Ok(_)  => {},
        Err(e) => println!("Error: {}", e),
    }
}

fn run(name: &String) -> Result<(), Box<Error>> {
    let file = try!(File::open(name));
    let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };

    let (module, format_player) = try!(format::load(&mmap[..]));
    println!("Title: {}", module.title);

    println!("Instruments:");
    for ins in &module.instrument {
        println!("{:3}: {:30} {:2}", ins.num, ins.name, ins.volume);
    }

    println!("Samples:");
    for smp in &module.sample {
        println!("{:3}: {:30} {:5} {:5} {:5} {}",
            smp.num, smp.name, smp.size, smp.loop_start, smp.loop_end,
            if smp.has_loop { 'L' } else { ' ' });
    }

    let mut player = player::Player::new(&module, format_player);

    println!("Length: {}", module.orders.num(0));
    println!("Patterns: {}", module.patterns.num());
    println!("Position: {} ({})", player.position(), module.orders.pattern(player.data.pos));

    show_pattern(&module, 0);

    let mut frame_info = FrameInfo::new();

    let file = try!(File::create("out.wav"));
    let writer = BufWriter::new(file);
    let mut wave_writer = try!(WaveWriter::new(2, 44100, 16, writer));

    for _ in 0..640 {
        let buffer = player.info(&mut frame_info).play_frame().buffer();
        println!("info pos:{} row:{} frame:{} speed:{} bpm:{}", frame_info.pos, frame_info.row, frame_info.frame, frame_info.speed, frame_info.bpm);
        println!("buffer {:?}", buffer);
        buffer.iter().for_each(|x| { wave_writer.write_sample_i16(*x); });
    }

    wave_writer.sync_header();

    Ok(())
}

fn show_pattern(module: &module::Module, num: usize) {
    println!("Pattern {}:", num);
    for r in 0..module.patterns.rows(num) {
        print!("{:3}: ", r);
        for c in 0..module.chn {
            print!("{}  ", module.patterns.event(num, r, c))
        }
        println!();
    }
}
