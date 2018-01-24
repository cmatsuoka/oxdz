use module::Module;
use ::*;

pub mod mk;
pub mod stm;

// Trait for module formats

pub trait Loader {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8]) -> Result<(), Error>;
    fn load(self: Box<Self>, &[u8]) -> Result<Module, Error>;
}


pub fn list() -> Vec<Box<Loader>> {
    vec![
        Box::new(mk::ModLoader),
        Box::new(stm::StmLoader),
    ]
}

pub fn load(b: &[u8]) -> Result<Module, Error> {

    for f in list() {
        println!("Probing format: {}", f.name());
        if f.probe(b).is_ok() {
            println!("Probe ok, load format");
            return f.load(b)
        }
    }

    Err(Error::Format("unsupported module format"))
}


/*
/// Standard order processor
///
/// Formats with non-standard order processing should implement their own
/// order processor.

pub struct StdOrders {
    rstpos: usize,
    orders: Vec<u8>,
    songs : Vec<u8>,  // vector of song entry points
}

impl StdOrders {
    fn from_slice(r: u8, o: &[u8]) -> Self {
        
        let mut r = r as usize;

        if r >= o.len() {
            r = 0;
        }

        StdOrders {
            rstpos: r,
            orders: o.to_vec(),
            songs : Vec::new(),
        }
    }

    fn num_patterns(&self) -> usize {
        let mut num = 0;
        self.orders.iter().for_each(|x| num = cmp::max(*x as usize, num));
        num + 1
    }
}

impl Orders for StdOrders {
    fn num(&self, song: usize) -> usize {
        self.orders.len()
    }

    fn restart_position(&mut self) -> usize {
        self.rstpos
    }

    fn pattern(&self, pos: usize) -> usize {
        self.orders[pos] as usize
    }

    fn next(&self, data: &mut PlayerData) -> usize {
        if data.pos < self.num(data.song) - 1 {
            data.pos += 1;
        }
        data.pos
    }

    fn prev(&self, data: &mut PlayerData) -> usize {
        if data.pos > 0 {
            data.pos -= 1;
        }
        data.pos
    }

    fn num_songs(&self) -> usize {
        self.songs.len()
    }

    fn next_song(&self, data: &mut PlayerData) -> usize {
        if data.song < self.num_songs() - 1 {
            data.song += 1;
        }
        data.song
    }

    fn prev_song(&self, data: &mut PlayerData) -> usize {
        if data.song > 0 {
            data.song -= 1;
        }
        data.song
    }
}
*/
