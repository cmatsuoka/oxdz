extern crate memmap;
extern crate oxdz;

use std::env;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use memmap::Mmap;
use oxdz::module::{self, event};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("usage: {} <filename> <num>", Path::new(&args[0]).file_name().unwrap().to_str().unwrap());
        return;
    }

    match run(args) {
        Ok(_)  => {},
        Err(e) => eprintln!("error: {}", e),
    }
}


fn run(args: Vec<String>) -> Result<(), Box<Error>> {

    let filename = &args[1];
    let num = parse_num(&args[2])?;

    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };

    let module = oxdz::format::load(&mmap[..], "")?;
    show_pattern(&module, num);

    Ok(())
}

fn parse_num(s: &str) -> Result<usize, std::num::ParseIntError> {
    if s.starts_with("0x") {
        usize::from_str_radix(&s[2..], 16)
    } else {
        s.parse()
    }
}

fn show_pattern(module: &module::Module, num: usize) {
    println!("Pattern {}:", num);
    let rows = module.rows(num);
    let ch = module.channels;
    let mut buffer = vec![0_u8; 6 * rows * ch];

    module.pattern_data(num, &mut buffer);

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
