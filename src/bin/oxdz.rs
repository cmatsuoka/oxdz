extern crate memmap;
extern crate oxdz;

use std::error::Error;
use std::fs::File;
use memmap::Mmap;
use oxdz::format;

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

    for ins in module.instrument {
        println!("{:3}: {}", ins.num, ins.name);
    }

    Ok(())
}
