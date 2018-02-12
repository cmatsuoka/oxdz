use module::Module;
use ::*;

pub mod mk;
pub mod st;
pub mod stm;
pub mod s3m;

// Supported formats

#[derive(PartialEq, Debug)]
pub enum Format {
    Mk,
    St,
    Xxch,
    S3m,
    Stm,
}

// Trait for module loader

pub trait Loader {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8], &str) -> Result<Format, Error>;
    fn load(self: Box<Self>, &[u8], Format) -> Result<Module, Error>;
}


pub fn list() -> Vec<Box<Loader>> {
    vec![
        Box::new(s3m::S3mLoader),
        Box::new(stm::StmLoader),
        Box::new(mk::ModLoader),
        Box::new(st::StLoader),
    ]
}

pub fn load(b: &[u8], player_id: &str) -> Result<Module, Error> {

    for f in list() {
        println!("Probing format: {}", f.name());

        let fmt = match f.probe(b, player_id) {
            Ok(val) => val,
            Err(_)  => continue,
        };

        println!("Probe ok, load format {:?}", fmt);
        return f.load(b, fmt)
    }

    Err(Error::Format("unsupported module format".to_owned()))
}
