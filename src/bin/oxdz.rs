extern crate memmap;
extern crate oxdz;

use std::error::Error;
use std::fs::File;
use memmap::Mmap;
use oxdz::{format, module, player};

fn main() {

    match run() {
        Ok(_)  => {},
        Err(e) => println!("Error: {}", e),
    }
}

fn run() -> Result<(), Box<Error>> {
    let file = try!(File::open("space_debris.mod"));
    let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };

    let module = try!(format::load_module(&mmap[..]));
    println!("Title: {}", module.title);

    println!("Instruments:");
    for ins in &module.instrument {
        println!("{:3}: {}", ins.num, ins.name);
    }

    println!("Samples:");
    for smp in &module.sample {
        println!("{:3}: {:30} {:5} {:5} {:5} {}",
            smp.num, smp.name, smp.size, smp.loop_start, smp.loop_end,
            if smp.has_loop { 'L' } else { ' ' });
    }

    let mut player = player::Player::new(&module);

    println!("Length: {}", module.orders.num(0));
    println!("Patterns: {}", module.patterns.num());
    println!("Position: {} ({})", player.position(), module.orders.pattern(&player));

    show_pattern(&module, 0);

    player.play_frame();
    player.play_frame();
    player.play_frame();
    player.play_frame();

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
