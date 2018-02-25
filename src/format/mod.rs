use module::Module;
use ::*;

pub mod mk;
pub mod st;
pub mod stm;
pub mod s3m;
pub mod xm;
pub mod fest;

// Supported formats

#[derive(PartialEq, Debug)]
pub enum Format {
    Mk,
    St,
    Ust,
    Xxch,
    Fest,
    Flt,
    S3m,
    Stm,
    Xm,
}

pub struct FormatInfo {
    pub format: Format,
    pub title : String,
}

// Trait for module loader

pub trait Loader {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8], &str) -> Result<FormatInfo, Error>;
    fn load(self: Box<Self>, &[u8], FormatInfo) -> Result<Module, Error>;
}


pub fn list() -> Vec<Box<Loader>> {
    vec![
        Box::new(s3m::S3mLoader),
        Box::new(stm::StmLoader),
        Box::new(mk::ModLoader),
        Box::new(st::StLoader),
        Box::new(fest::FestLoader),
    ]
}

pub fn load(b: &[u8], player_id: &str) -> Result<Module, Error> {

    for f in list() {
        println!("Probing format: {}", f.name());

        let info = match f.probe(b, player_id) {
            Ok(val) => val,
            Err(_)  => continue,
        };

        println!("Probe ok, load format {:?}", info.format);
        return f.load(b, info)
    }

    Err(Error::Format("unsupported module format".to_owned()))
}
