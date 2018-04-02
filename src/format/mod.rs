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
    Xchn,
    Xxch,
    Fest,
    Flt,
    S3m,
    Stm,
    Xm,
}

pub struct ProbeInfo {
    pub format: Format,
    pub title : String,
}

pub struct FormatInfo {
    pub name: &'static str,
}

// Trait for module loader

pub trait Loader: Sync {
    fn name(&self) -> &'static str;
    fn probe(&self, &[u8], &str) -> Result<ProbeInfo, Error>;
    fn load(self: Box<Self>, &[u8], ProbeInfo) -> Result<Module, Error>;
}

fn loader_list() -> Vec<Box<Loader>> {
    vec![
        Box::new(xm::XmLoader),
        Box::new(s3m::S3mLoader),
        Box::new(stm::StmLoader),
        Box::new(mk::ModLoader),
        Box::new(st::StLoader),
        Box::new(fest::FestLoader),
    ]
}

pub fn list() -> Vec<FormatInfo> {
    loader_list().iter().map(|x| FormatInfo{name: x.name()}).collect()
}

pub fn load(b: &[u8], player_id: &str) -> Result<Module, Error> {

    for f in loader_list() {
        debug!("Probing format: {}", f.name());

        let info = match f.probe(b, player_id) {
            Ok(val) => val,
            Err(_)  => continue,
        };

        debug!("Probe ok, load format {:?}", info.format);
        return f.load(b, info)
    }

    Err(Error::Format("unsupported module format".to_owned()))
}
