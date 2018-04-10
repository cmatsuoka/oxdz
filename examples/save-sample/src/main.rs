extern crate memmap;
extern crate oxdz;

#[macro_use]
extern crate quick_error;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use memmap::Mmap;

quick_error! {
    #[derive(Debug)]
    enum MyError {
        InvalidSample(num: usize) {
            description("Invalid sample number")
            display("Sample {} is invalid", num)
        }
    }
}


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

    let mut module = oxdz::format::load(&mmap[..], "")?;
    let samples = module.data.samples();

    if num >= samples.len() {
        return Err(Box::new(MyError::InvalidSample(num)));
    }

    let out_filename = format!("sample_{}.raw", num);
    let file = File::create(out_filename)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(samples[num].data_u8())?;

    Ok(())
}

fn parse_num(s: &str) -> Result<usize, std::num::ParseIntError> {
    if s.starts_with("0x") {
        usize::from_str_radix(&s[2..], 16)
    } else {
        s.parse()
    }
}
