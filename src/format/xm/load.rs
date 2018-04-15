use format::{ProbeInfo, Format, Loader};
use format::xm::{XmData, SongHeaderTyp, InstrHeaderTyp, SampleHeaderTyp, PatternHeaderTyp};
use module::{Module, Sample};
use module::sample::SampleType;
use util::{BinaryRead, SliceConvert};
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
        let patterns: Vec<PatternHeaderTyp>;
        let instruments: Vec<InstrHeaderTyp>;
        let mut samples: Vec<Sample>;

        if version >= 0x0104 {
            patterns = load_patterns(&header, &b, &mut offset)?;
            let ins_list = load_instruments(&header, &b, &mut offset)?;
            instruments = ins_list.0;
            samples = ins_list.1;
        } else {
            let ins_list = load_instruments(&header, &b, &mut offset)?;
            patterns = load_patterns(&header, &b, &mut offset)?;
            instruments = ins_list.0;
            samples = ins_list.1;

            // XM 1.03 stores all samples after the patterns
            let mut smp_num = 1;
            for ins in &instruments {
                for samp in &ins.samp {
                    let smp = load_sample(&samp, smp_num, b, &mut offset)?;
                    samples.push(smp);
                    smp_num += 1;
                }
            }
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

fn load_patterns(header: &SongHeaderTyp, b: &[u8], offset: &mut usize) -> Result<Vec<PatternHeaderTyp>, Error> {
    let mut patterns = Vec::with_capacity(header.ant_ptn as usize);
    for i in 0..header.ant_ptn as usize - 1 {
        let ptn = PatternHeaderTyp::from_slice(b.slice(*offset, b.len() - *offset)?, header.ver, header.ant_chn as usize)?;
        debug!("pattern {}: {} rows", i, ptn.patt_len);
        *offset += ptn.pattern_header_size as usize + ptn.data_len as usize;
        patterns.push(ptn);
    }
    // alloc one extra pattern
    patterns.push(PatternHeaderTyp::new_empty(header.ant_chn as usize));

    Ok(patterns)
}

fn load_instruments(header: &SongHeaderTyp, b: &[u8], mut offset: &mut usize) -> Result<(Vec<InstrHeaderTyp>, Vec<Sample>), Error> {
    let mut instruments: Vec<InstrHeaderTyp> = Vec::with_capacity(header.ant_instrs as usize);
    let mut samples: Vec<Sample> = Vec::new();
    let mut smp_num = 1;

    for _i in 0..header.ant_instrs as usize {
        let ins = InstrHeaderTyp::from_slice(smp_num, b.slice(*offset, b.len() - *offset)?)?;
        let ant_samp = ins.ant_samp;

        *offset += ins.instr_size as usize + 40 * ant_samp as usize;
        if ant_samp > 0 {

            for j in 0..ant_samp as usize {
                if header.ver >= 0x0104 {
                    let smp = load_sample(&ins.samp[j], smp_num, b, &mut offset)?;
                    samples.push(smp);
                }
                smp_num += 1;
            }
        }

        instruments.push(ins);
    }

    Ok((instruments, samples))
}

fn load_sample(samp: &SampleHeaderTyp, smp_num: usize, b: &[u8], offset: &mut usize) -> Result<Sample, Error> {
    let mut smp = Sample::new();
    smp.num = smp_num;
    smp.name = samp.name.to_owned();
    smp.size = samp.len as u32;
    let byte_size = samp.len as usize;
    smp.sample_type = if samp.typ & 16 != 0 {
        let buf = diff_decode_16l(b.slice(*offset, byte_size)?);
        smp.store(&buf[..].as_slice_u8());
        SampleType::Sample16
    } else {
        let buf = diff_decode_8(b.slice(*offset, byte_size)?);
        smp.store(&buf[..]);
        SampleType::Sample8
    };
    *offset += byte_size;

    Ok(smp)
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

fn diff_decode_16l(b: &[u8]) -> Vec<u16> {
    let mut buf: Vec<u16> = vec![0; b.len()];
    let mut old = 0_u16;
    for i in 0..b.len() / 2 {
        let val = ((b[i*2+1] as u16) << 8) + b[i*2] as u16;
        let new = val.wrapping_add(old);
        buf[i] = new;
        old = new;
    }
    buf
}
