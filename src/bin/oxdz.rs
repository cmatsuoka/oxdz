extern crate memmap;
extern crate oxdz;
extern crate riff_wave;
extern crate getopts;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use getopts::Options;
use memmap::Mmap;
use oxdz::{format, player, FrameInfo};
use oxdz::module::{self, event};
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

    let mut module = try!(format::load(&mmap[..], ""));
    println!("Title: {}", module.title());

    println!("Instruments:");
    let mut i = 0;
    for ins in module.instruments() {
        println!("{:3}: {:30}", i + 1, ins);
        i += 1;
    }

    println!("Samples:");
    for smp in module.samples() {
        println!("{:3}: {:30} {:5}", smp.num, smp.name, smp.size);
    }

    println!("Default player for this format: {}", module.player);
    println!("Available players:");
    for p in player::list() {
        println!("{:5} {:40} {:?}", p.id, p.name, p.accepts);
    }

    let list_entry = player::list_by_id(module.player)?;
    module = list_entry.import(module)?;

    let mut player = player::Player::find(&module, module.player, "")?;

    println!("Length: {}", module.len());
    println!("Patterns: {}", module.patterns());
    println!("Position: {} ({})", player.position(), module.pattern_in_position(player.data.pos).unwrap());

    show_pattern(&module, 0);

    let mut frame_info = FrameInfo::new();

    let file = try!(File::create("out.wav"));
    let writer = BufWriter::new(file);
    let mut wave_writer = try!(WaveWriter::new(2, 44100, 16, writer));

    player.start();
    for _ in 0..1000 {
        let buffer = player.info(&mut frame_info).play_frame().buffer();
        print!("info pos:{} row:{} frame:{} speed:{} tempo:{}    \r", frame_info.pos, frame_info.row, frame_info.frame, frame_info.speed, frame_info.tempo);
        for s in buffer {
            try!(wave_writer.write_sample_i16(*s));
        }
    }
    println!();

    try!(wave_writer.sync_header());

    Ok(())
}

fn show_pattern(module: &module::Module, num: usize) {
    println!("Pattern {}:", num);
    let rows = module.rows(num);
    let ch = module.channels();
    let mut buffer = vec![0_u8; 6 * rows * ch];

    module.pattern_data(0, &mut buffer);

    let mut ofs = 0;
    for r in 0..rows {
        print!("{:3}: ", r);
        for _ in 0..ch {
            print!("{}  ", event::format(&buffer[ofs..ofs+6]));
            ofs += 6;
        }
        println!();
    }
}
