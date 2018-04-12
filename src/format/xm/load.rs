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
        let mut samples: Vec<Sample> = Vec::new();
        let mut smp_num = 1;

        for i in 0..header.ant_instrs as usize {
            let ins = InstrHeaderTyp::from_slice(smp_num, b.slice(offset, b.len() - offset)?)?;
            let ant_samp = ins.ant_samp;

            offset += ins.instr_size as usize + 40 * ant_samp as usize;
            if ant_samp > 0 {

                for j in 0..ant_samp as usize {
                    let samp = &ins.samp[j];
                    let mut smp = Sample::new();
                    smp.num = smp_num;
                    smp.name = samp.name.to_owned();
                    smp.size = samp.len as u32;
                    let byte_size: usize;
                    smp.sample_type = if samp.typ & 4 != 0 {
                        byte_size = samp.len as usize * 2;
                        smp.store(b.slice(offset, byte_size)?);
                        SampleType::Sample16
                    } else {
                        byte_size = samp.len as usize;
                        let buf = diff_decode_8(b.slice(offset, byte_size)?);
                        //let buf = b.slice(offset, byte_size)?;
                        smp.store(&buf[..]);
                        SampleType::Sample8
                    };

                    smp_num += 1;
                    offset += byte_size;
                    samples.push(smp);
                }
            }

            instruments.push(ins);
        }

        let data = XmData{
            header,
            patterns,
            instruments,
            samples,
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


fn diff_decode_8(b: &[u8]) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![0; b.len()];
    let mut old = 0_u8;
    for i in 0..b.len() {
        let new = b[i].wrapping_add(old);
        buf[i] = new;
        old = new;
    }
    buf
}

