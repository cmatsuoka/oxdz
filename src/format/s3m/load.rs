use format::Loader;
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

impl S3mLoader {
    fn load_instrument(&self, b: &[u8], i: usize, ofs: usize, cvt: bool) -> Result<(S3mInstrument, Sample), Error> {
        let mut smp = Sample::new();
        let flags = b.read8(ofs)?;
        let typ = b.read8(ofs)?;
        let vol = b.read8(ofs + 0x1c)?;

        let c2spd      = 8363; // b.read16l_lo_hi(ofs + 0x20)?;
        smp.name       = b.read_string(ofs + 0x30, 28)?;
        smp.size       = b.read16l_lo_hi(ofs + 0x10)? as usize;
        smp.loop_start = b.read16l_lo_hi(ofs + 0x14)? as usize;
        smp.loop_end   = b.read16l_lo_hi(ofs + 0x18)? as usize;
        smp.rate       = c2spd as f64;
        smp.num        = i + 1;
        smp.has_loop   = flags & 0x01 != 0;

        if smp.loop_end == 0xffff {
            smp.loop_end = 0;
        }

        if smp.size > 0 {
            smp.sample_type = if flags & 0x04 != 0 { SampleType::Sample16 } else { SampleType::Sample8 };
        }

        smp.sanity_check();
        let sample_offset = b.read16l(ofs + 0x0e)? as usize * 16;
        let sample_size = if flags & 0x04 != 0 { smp.size*2 } else { smp.size };
        smp.store(b.slice(sample_offset, sample_size)?);
        if cvt {
            smp.to_signed();
        }

        let ins = S3mInstrument {
            typ,
            c2spd,
            vol: vol as i8,
        };

        Ok((ins, smp))
    }
}

impl Loader for S3mLoader {
    fn name(&self) -> &'static str {
        "Scream Tracker 3 S3M"
    }
  
    fn probe(&self, b: &[u8]) -> Result<(), Error> {
        if b.len() < 256 {
            return Err(Error::Format("file too short"));
        }

        let typ = b.read8(0x1d)?;
        let magic = b.read_string(0x2c, 4)?;
        if typ == 16 && magic == "SCRM" {
            Ok(())
        } else {
            Err(Error::Format("bad magic"))
        }
    }

    fn load(self: Box<Self>, b: &[u8]) -> Result<Module, Error> {
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
        let mut instrum_pp = Vec::<usize>::new();
        for _ in 0..ins_num { instrum_pp.push(b.read16l(ofs)? as usize * 16); ofs += 2; }

        // Pattern parapointers
        let mut pattern_pp = Vec::<usize>::new();
        for _ in 0..pat_num { pattern_pp.push(b.read16l(ofs)? as usize * 16); ofs += 2; }
 
        // Channel pan positions
        let ch_pan = b.slice(ofs, 32)?;

        // Load instruments
        let mut instruments = Vec::<S3mInstrument>::new();
        let mut samples = Vec::<Sample>::new();
        for i in 0..ins_num as usize {
            let (ins, smp) = try!(self.load_instrument(b, i, instrum_pp[i], ffi != 1));
            instruments.push(ins);
            samples.push(smp);
        }

        // Load patterns
        let mut patterns = Vec::<S3mPattern>::new();
        for i in 0..pat_num as usize {
            let ofs = pattern_pp[i];
            let plen = b.read16l(ofs)? as usize;
            patterns.push(S3mPattern{ size: plen, data: b.slice(ofs + 2, plen)?.to_vec() });
        }

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
            player     : "st3",
            data       : Box::new(data),
        };

        Ok(m)
    }
}

