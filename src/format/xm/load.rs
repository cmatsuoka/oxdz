use format::{ProbeInfo, Format, Loader};
use format::xm::{XmData, SongHeaderTyp, InstrHeaderTyp, SampleHeaderTyp, PatternHeaderTyp};
use module::{Module, Sample};
use module::sample::SampleType;
use util::BinaryRead;
use ::*;

/// FastTracker 2 module loader
pub struct XmLoader;

impl Loader for XmLoader {
    fn name(&self) -> &'static str {
        "FastTracker II XM"
    }
  
    fn probe(&self, b: &[u8], player_id: &str) -> Result<ProbeInfo, Error> {
        if b.len() < 128 {
            return Err(Error::Format(format!("file too short ({})", b.len())));
        }

        let magic = b.read_string(0, 17)?;
        if magic == "Extended Module: " {
            player::check_accepted(player_id, "xm")?;
            Ok(ProbeInfo{format: Format::Xm, title: b.read_string(17, 20)?})
        } else {
            Err(Error::Format("bad magic".to_owned()))
        }
    }

    fn load(self: Box<Self>, b: &[u8], info: ProbeInfo) -> Result<Module, Error> {

        if info.format != Format::Xm {
            return Err(Error::Format("unsupported format".to_owned()));
        }

        let header = SongHeaderTyp::from_slice(&b)?;
        let version = header.ver;
        let creator = header.prog_name.clone();
   	let channels = header.ant_chn as usize;

        let mut offset = 60 + header.header_size as usize;
        let mut patterns: Vec<PatternHeaderTyp> = Vec::with_capacity(header.ant_ptn as usize);
        for i in 0..header.ant_ptn as usize {
            let ptn = PatternHeaderTyp::from_slice(i, b.slice(offset, b.len() - offset)?, header.ant_chn as usize)?;
            debug!("pattern {}: {} rows", i, ptn.patt_len);
            offset += ptn.pattern_header_size as usize + ptn.data_len as usize;
            patterns.push(ptn);
        }

        let mut instruments: Vec<InstrHeaderTyp> = Vec::with_capacity(header.ant_instrs as usize);
        for i in 0..header.ant_instrs as usize {
            let (ins, size) = InstrHeaderTyp::from_slice(b.slice(offset, b.len() - offset)?)?;
            offset += size;
            instruments.push(ins);
        }

        let data = XmData{
            header,
            patterns,
            instruments,
            samples: Vec::new(),
        };

        let m = Module {
            format_id  : "xm",
            description: format!("Extended module v{}.{:02}", version >> 8, version & 0x0ff),
            creator,
            channels,
            player     : "ft2",
            data       : Box::new(data),
        };

        Ok(m)
    }
}

