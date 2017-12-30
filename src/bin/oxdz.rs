extern crate oxdz;

use oxdz::format;

fn main() {
    for f in format::list() {
        println!("format: {}", f.name());
    }
}
