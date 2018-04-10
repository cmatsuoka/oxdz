use format::{ProbeInfo, Loader, Format};
use format::s3m::{S3mData, S3mPattern, S3mInstrument};
use module::{Module, Sample};
use module::sample::SampleType;
use util::BinaryRead;
use ::*;

pub trait BinaryReadExt {
    fn read16l_lo_hi(&self, ofs: usize) -> Result<u32, Error>;
}

impl<'a> BinaryReadExt for &'a [u8] {
    fn read16l_lo_hi(&self, ofs: usize) -> Result<u32, Error> {
        let lo = self.read16l(ofs)? as u32;
        let hi = self.read16l(ofs + 2)? as u32;
        Ok((hi << 16) | lo)
    }

}

/// Scream Tracker 2 module loader
pub struct S3mLoader;

impl Loader for S3mLoader {
    fn name(&self) -> &'static str {
        "Scream Tracker 3"
    }

    fn probe(&self, b: &[u8], player_id: &str) -> Result<ProbeInfo, Error> {
        if b.len() < 256 {
            return Err(Error::Format(format!("file too short ({})", b.len())));
        }

        let typ = b.read8(0x1d)?;
        let magic = b.read_string(0x2c, 4)?;
        if typ == 16 && magic == "SCRM" {
            // is S3M
            player::check_accepted(player_id, "s3m")?;
            Ok(ProbeInfo{format: Format::S3m, title: b.read_string(0, 28)?})
        } else {
            Err(Error::Format(format!("bad magic {:?}", magic)))
        }
    }

    fn load(self: Box<Self>, b: &[u8], info: ProbeInfo) -> Result<Module, Error> {

        if info.format != Format::S3m {
            return Err(Error::Format("unsupported format".to_owned()));
        }

        let song_name = b.read_string(0, 28)?;
        let ord_num = b.read16l(0x20)?;
        let ins_num = b.read16l(0x22)?;
        let pat_num = b.read16l(0x24)?;
        let flags = b.read16l(0x26)?;
        let cwt_v = b.read16l(0x28)?;
        let ffi = b.read16l(0x2a)?;
        let g_v = b.read8(0x30)?;
        let i_s = b.read8(0x31)?;
        let i_t = b.read8(0x32)?;
        let m_v = b.read8(0x33)?;
        let d_p = b.read8(0x35)?;
        let ch_settings = b.slice(0x40, 32)?;

        // Orders
        let orders = b.slice(0x60, ord_num as usize)?.to_vec();

        // Instrument parapointers
        let mut ofs = 0x60_usize + ord_num as usize;
        let mut instrum_pp = Vec::<u32>::new();
        for _ in 0..ins_num { instrum_pp.push(b.read16l(ofs)? as u32 * 16); ofs += 2; }

        // Pattern parapointers
        let mut pattern_pp = Vec::<u32>::new();
        for _ in 0..pat_num { pattern_pp.push(b.read16l(ofs)? as u32 * 16); ofs += 2; }

        // Channel pan positions
        let ch_pan = b.slice(ofs, 32)?;

        // Load instruments
        let mut instruments = Vec::<S3mInstrument>::new();
        let mut samples = Vec::<Sample>::new();
        for i in 0..ins_num as usize {
            let ins = load_instrument(b, instrum_pp[i] as usize)?;
            let smp = load_sample(b, i, &ins, ffi != 1)?;
            instruments.push(ins);
            samples.push(smp);
        }

        // Load patterns
        let mut patterns = Vec::<S3mPattern>::new();
        for i in 0..pat_num as usize {
            let ofs = pattern_pp[i] as usize;
            let plen = b.read16l(ofs)? as usize;
            patterns.push(S3mPattern{ size: plen, data: b.slice(ofs, plen + 2)?.to_vec() });
        }

        let num_chn = {
            let mut chn = 0;
            for i in 0..32 {
                if ch_settings[i] == 0xff {
                    continue
                }
                chn = i
            }
            chn + 1
        };

        let mut data = S3mData{
            song_name,
            ord_num,
            ins_num,
            pat_num,
            flags,
            cwt_v,
            ffi,
            g_v,
            m_v,
            i_s,
            i_t,
            d_p,
            ch_settings: [0; 32],
            orders,
            instrum_pp,
            pattern_pp,
            ch_pan: [0; 32],
            instruments,
            patterns,
            samples,

            channels: num_chn,
        };

        data.ch_settings.copy_from_slice(ch_settings);
        data.ch_pan.copy_from_slice(ch_pan);

        let ver_major = (cwt_v & 0xf00) >> 8;
        let ver_minor = cwt_v & 0x0ff;

        let m = Module {
            format_id  : "s3m",
            description: format!("Scream Tracker 3 S3M"),
            creator    : match cwt_v >> 12 {
                             1 => format!("Scream Tracker {}.{:02x}", ver_major, ver_minor),
                             2 => format!("Imago Orpheus {}.{:02x}", ver_major, ver_minor),
                             3 => match cwt_v {
                                      0x3216 => "Impulse Tracker 2.14v3".to_owned(),
                                      0x3217 => "Impulse Tracker 2.14v5".to_owned(),
                                      _      => format!("Impulse Tracker {}.{:02x}", ver_major, ver_minor),
                             },
                             4 => if cwt_v != 0x4100 {
                                      format!("Schism Tracker {}.{:02x}", ver_major, ver_minor)
                                  } else {
                                      "BeRoTracker 1.00".to_owned()
                                  },
                             5 => format!("OpenMPT {}.{:02x}", ver_major, ver_minor),
                             6 => format!("BeRoTracker {}.{:02x}", ver_major, ver_minor),
                             _ => format!("unknown ({}.{:02x}", ver_major, ver_minor),
                         },
            channels   : num_chn,
            player     : "st3",
            data       : Box::new(data),
        };

        Ok(m)
    }
}

fn load_instrument(b: &[u8], ofs: usize) -> Result<S3mInstrument, Error> {
    let mut ins = S3mInstrument::new();

    ins.typ      = b.read8(ofs)?;
    ins.memseg   = (b.read16l(ofs + 0x0e)? as u32) | ((b.read8(ofs + 0x0d)? as u32) << 16);
    ins.length   = b.read16l_lo_hi(ofs + 0x10)?;
    ins.loop_beg = b.read16l_lo_hi(ofs + 0x14)?;
    ins.loop_end = b.read16l_lo_hi(ofs + 0x18)?;
    ins.vol      = b.read8i(ofs + 0x1c)?;
    ins.flags    = b.read8i(ofs + 0x1f)?;
    ins.c2spd    = b.read16l_lo_hi(ofs + 0x20)?;
    ins.name     = b.read_string(ofs + 0x30, 28)?;

    Ok(ins)
}

fn load_sample(b: &[u8], i: usize, ins: &S3mInstrument, cvt: bool) -> Result<Sample, Error> {
    let mut smp = Sample::new();

    smp.num  = i + 1;
    smp.address = (ins.memseg as u32) << 4;
    smp.name = ins.name.to_owned();
    smp.size = ins.length;

    if smp.size > 0 {
        smp.sample_type = if ins.flags & 0x04 != 0 { SampleType::Sample16 } else { SampleType::Sample8 };
    }

    let sample_size = if ins.flags & 0x04 != 0 { smp.size*2 } else { smp.size };
    smp.store(b.slice((ins.memseg as usize) << 4, sample_size as usize)?);
    if cvt {
        smp.to_signed();
    }

    Ok(smp)
}
