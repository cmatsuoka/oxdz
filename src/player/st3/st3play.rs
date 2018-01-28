use module::{Module, ModuleData};
use player::{PlayerData, FormatPlayer};
use format::s3m::S3mData;
use mixer::Mixer;

/// S3M replayer
///
/// Based on ST3PLAY v0.78 - 9th of February - http://16-bits.org
/// St3play is a very accurate C port of Scream Tracker 3.21's replayer,
/// by Olav "8bitbubsy" Sørensen, based on the original asm source codes
/// by Sami "PSI" Tammilehto (Future Crew).

enum SoundCard {
    Sb,
    Gus,
}

// TRACKER ID
const SCREAM_TRACKER : u8 = 1;
const IMAGO_ORPHEUS  : u8 = 2;
const IMPULSE_TRACKER: u8 = 3;
const SCHISM_TRACKER : u8 = 4;
const OPENMPT        : u8 = 5;
const BEROTRACKER    : u8 = 6;
// there is also CREAMTRACKER (7), but let's ignore that for now

#[derive(Default)]
struct Chn {
    aorgvol       : i8,
    avol          : i8,
    channelnum    : u8,
    achannelused  : u8,
    aglis         : u8,
    atremor       : u8,
    atreon        : u8,
    atrigcnt      : u8,
    anotecutcnt   : u8,
    anotedelaycnt : u8,
    avibtretype   : u8,
    note          : u8,
    ins           : u8,
    vol           : u8,
    cmd           : u8,
    info          : u8,
    lastins       : u8,
    lastnote      : u8,
    alastnfo      : u8,
    alasteff      : u8,
    alasteff1     : u8,
    apanpos       : i16,
    avibcnt       : i16,
    astartoffset  : u16,
    astartoffset00: u16,
    ac2spd        : i32,
    asldspd       : i32,
    aorgspd       : i32,
    aspd          : i32,

    // NON-ST3 variables
    chanvol       : i8,
    surround      : u8,
    apantype      : u8,
    nxymem        : u8,
    pxymem        : u8,
    txxmem        : u8,
    wxymem        : u8,
    yxymem        : u8,
    apancnt       : i16,
}

#[derive(Default)]
pub struct St3Play {
    // STATIC DATA
    tickdelay         : i8,  // NON-ST3
    volslidetype      : i8,
    patterndelay      : i8,
    patloopcount      : i8,
    breakpat          : u8,
    startrow          : u8,
    musiccount        : u8,
    np_ord            : usize,  // i16,
    np_row            : i16,
    np_pat            : i16,
    np_patoff         : i16,
    patloopstart      : i16,
    jumptorow         : i16,
    patternadd        : u16,
    patmusicrand      : u16,
    aspdmax           : i32,
    aspdmin           : i32,
    np_patseg         : usize,  // u32,
    chn               : [Chn; 32],
    soundcardtype     : u8,
    soundBufferSize   : i32,
    audioFreq         : u32,
    //VOICE voice[32];
    //WAVEFORMATEX wfx;
    //WAVEHDR waveBlocks[MIX_BUF_NUM];
    //HWAVEOUT _hWaveOut;
    //float f_audioFreq;
    //float f_masterVolume;
    //samplingInterpolation: i8,
    //float *masterBufferL;
    //float *masterBufferR;
    //*mixerBuffer : i8,
    //samplesLeft : i32,
    //samplesPerFrame : u32,
    //volatile mixingMutex : i8,
    //volatile isMixing : i8,

    /* GLOBAL VARIABLES */
    //ModuleLoaded : i8,
    //MusicPaused : i8,
    //Playing : i8,

    //instrumentadd : u16,
    lastachannelused : u8, // i8,
    tracker : u8,
    oldstvib : bool,
    fastvolslide : bool,
    amigalimits : bool,
    musicmax : u8,
    numChannels : u8,
    tempo : i16,
    globalvol : i16,
    stereomode : i8,
    mastervol : u8,
    mseg_len : u32,
} 

/* TABLES */
static XFINETUNE_AMIGA: &'static [i16; 16] = &[
    7895, 7941, 7985, 8046, 8107, 8169, 8232, 8280,
    8363, 8413, 8463, 8529, 8581, 8651, 8723, 8757
];

static RETRIGVOLADD: &'static [i8; 32] = &[
    0, -1, -2, -4, -8,-16,  0,  0,
    0,  1,  2,  4,  8, 16,  0,  0,
    0,  0,  0,  0,  0,  0, 10,  8,
    0,  0,  0,  0,  0,  0, 24, 32
];

static NOTESPD: &'static [u16; 12] = &[
    1712 * 16, 1616 * 16, 1524 * 16,
    1440 * 16, 1356 * 16, 1280 * 16,
    1208 * 16, 1140 * 16, 1076 * 16,
    1016 * 16,  960 * 16,  907 * 16
];

static VIBSIN: &'static [i16; 64] = &[
     0x00, 0x18, 0x31, 0x4A, 0x61, 0x78, 0x8D, 0xA1,
     0xB4, 0xC5, 0xD4, 0xE0, 0xEB, 0xF4, 0xFA, 0xFD,
     0xFF, 0xFD, 0xFA, 0xF4, 0xEB, 0xE0, 0xD4, 0xC5,
     0xB4, 0xA1, 0x8D, 0x78, 0x61, 0x4A, 0x31, 0x18,
     0x00,-0x18,-0x31,-0x4A,-0x61,-0x78,-0x8D,-0xA1,
    -0xB4,-0xC5,-0xD4,-0xE0,-0xEB,-0xF4,-0xFA,-0xFD,
    -0xFF,-0xFD,-0xFA,-0xF4,-0xEB,-0xE0,-0xD4,-0xC5,
    -0xB4,-0xA1,-0x8D,-0x78,-0x61,-0x4A,-0x31,-0x18
];

static VIBSQU: &'static [u8; 64] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];

static VIBRAMP: &'static [i16; 64] = &[
       0, -248,-240,-232,-224,-216,-208,-200,
    -192, -184,-176,-168,-160,-152,-144,-136,
    -128, -120,-112,-104, -96, -88, -80, -72,
     -64,  -56, -48, -40, -32, -24, -16,  -8,
       0,    8,  16,  24,  32,  40,  48,  56,
      64,   72,  80,  88,  96, 104, 112, 120,
     128,  136, 144, 152, 160, 168, 176, 184,
     192,  200, 208, 216, 224, 232, 240, 248
];


// CODE START

impl St3Play {
    pub fn new(module: &Module) -> Self {
        Default::default()
    }

    fn getlastnfo(&self, ch: &mut Chn) {
        if ch.info != 0 {
            ch.info = ch.alastnfo;
        }
    }

    fn setspeed(&mut self, val: u8) {
        if val != 0 {
            self.musicmax = val;
        }
    }

    fn settempo(&mut self, val: u16) {
        if val > 32 {
            self.tempo = val as i16;
            //self.setSamplesPerFrame(((audioFreq * 5) / 2) / tempo);
        }
    }

    fn setspd(&mut self, ch: usize, mixer: &mut Mixer) {
        self.chn[ch].achannelused |= 0x80;
        let mut tmpspd = self.chn[ch].aspd;

        if self.amigalimits {
            if self.chn[ch].aorgspd > self.aspdmax {
                self.chn[ch].aorgspd = self.aspdmax;
            }
            if self.chn[ch].aorgspd < self.aspdmin {
                self.chn[ch].aorgspd = self.aspdmin;
            }
            if self.chn[ch].aspd > self.aspdmax {
                self.chn[ch].aspd = self.aspdmax;
            }
        }

        if self.tracker == SCREAM_TRACKER || self.amigalimits {
            if tmpspd > self.aspdmax {
                tmpspd = self.aspdmax;
            }
        } else {
            // *ABSOLUTE* max!
            if tmpspd > 14317056 {
                tmpspd = 14317056;
            }
        }

        if tmpspd == 0 {
            // cut channel
            //self.voice_set_sampling_frequency(ch, 0);
            return;
        }

        if tmpspd < self.aspdmin {
            tmpspd = self.aspdmin;

            if self.amigalimits && self.chn[ch].aspd < self.aspdmin {
                self.chn[ch].aspd = self.aspdmin;
            }
        }

        // ST3 actually uses 14317056 (3.579264MHz * 4) instead of 14317456 (1712*8363)
        if tmpspd > 0 {
            //self.voice_set_sampling_frequency(ch, 14317056 / tmpspd as u32);
            mixer.set_period(ch, tmpspd as f64 / 4.0);
        }
    }

    fn setvol(&mut self, ch: usize, mixer: &mut Mixer) {
        self.chn[ch].achannelused |= 0x80;
        mixer.set_volume(ch, ((self.chn[ch].avol as f64 / 63.0) * (self.chn[ch].chanvol as f64 / 64.0) *
                              (self.globalvol as f64 / 64.0) * 1024.0) as usize);
        mixer.set_pan(ch, self.chn[ch].apanpos as isize);
    }

    fn setpan(&mut self, ch: usize, mut mixer: &mut Mixer) {
        self.setvol(ch, &mut mixer);
    }

    fn stnote2herz(&mut self, note: u8) -> u16 {
        if note == 254 {
            return 0;
        }

        let mut tmpnote = note & 0x0F;
        let mut tmpocta = note >> 4;

        // ST3 doesn't do this, but do it for safety
        if tmpnote > 11 {
            tmpnote = 11;
        }

        // limit octaves to 8 in ST3 mode
        if self.tracker == SCREAM_TRACKER && tmpocta > 7 {
            tmpocta = 7;
        }

        return NOTESPD[tmpnote as usize] >> tmpocta;
    }

    fn scalec2spd(&mut self, ch: usize, spd: i32) -> i32 {
        let mut spd = spd * 8363;

        if self.tracker == SCREAM_TRACKER {
            if spd / 65536 >= self.chn[ch].ac2spd {
                return 32767;
            }
        }

        if self.chn[ch].ac2spd != 0 {
            spd /= self.chn[ch].ac2spd;
        }

        if self.tracker == SCREAM_TRACKER {
            if spd > 32767 {
                return 32767;
            }
        }

        return spd;
    }

    /* for Gxx with semitones slide flag */
    fn roundspd(&mut self, ch: usize, spd: i32) -> i32 {
        /*int8_t octa;
        int8_t lastnote;
        int8_t newnote;
        int32_t note;
        int32_t lastspd;
        int32_t newspd;*/

        let mut newspd = spd * self.chn[ch].ac2spd;

        if self.tracker == SCREAM_TRACKER {
            if newspd / 65536 >= 8363 {
                return spd;
            }
        }

        newspd /= 8363;

        // find octave
        let mut octa    = 0;
        let mut lastspd = (1712*8 + 907*16) / 2;
        while lastspd >= newspd {
            octa += 1;
            lastspd /= 2;
        }

        // find note
        let mut lastnote = 0;
        let mut newnote  = 0;

        lastspd = if self.tracker == SCREAM_TRACKER { 32767 } else { 32767 * 2 };

        while newnote < 11 {
            let mut note = (NOTESPD[newnote] >> octa) as i32 - newspd;
            if note < 0 {
                note *= -1; /* abs() */
            }

            if note < lastspd {
                lastspd  = note;
                lastnote = newnote;
            }

            newnote += 1;
        }

        // get new speed from new note
        newspd = ((self.stnote2herz((octa << 4) | (lastnote & 0x0F) as u8)) * 8363) as i32;

        if self.tracker == SCREAM_TRACKER {
            if (newspd / 65536) >= self.chn[ch].ac2spd {
                return spd;
            }
        }

        if self.chn[ch].ac2spd != 0 {
            newspd /= self.chn[ch].ac2spd;
        }

        return newspd;
    }

    fn neworder(&mut self, module: &S3mData) -> i16 {
        loop {
            self.np_ord += 1;

            if module.orders[self.np_ord - 1] == 255 || self.np_ord > module.ord_num as usize {  // end
                self.np_ord = 1;
            }

            if module.orders[self.np_ord - 1] == 254 {  // skip
                continue;  // goto newOrderSkip;
            }

            self.np_pat       = module.orders[self.np_ord - 1] as i16;
            self.np_patoff    = -1;  // force reseek
            self.np_row       = self.startrow as i16;
            self.startrow     = 0;
            self.patmusicrand = 0;
            self.patloopstart = -1;
            self.jumptorow    = -1;

            return self.np_row;
        }
    }

    // updates np_patseg and np_patoff
    fn seekpat(&mut self, module: &S3mData) {
        if self.np_patoff == -1 {  // seek must be done
            self.np_patseg = module.pattern_pp[self.np_pat as usize] * 16;
            if self.np_patseg != 0 {
                let mut j = 2;  // skip packed pat len flag
    
                // increase np_patoff on patbreak
                if self.np_row != 0 {
                    let mut i = self.np_row;
                    while i != 0 {
                        let dat = module.patterns[self.np_patseg].data[j]; j += 1;
                        if dat == 0 {
                            i -= 1;
                        } else {
                            // skip ch data
                            if dat & 0x20 != 0 { j += 2 }
                            if dat & 0x40 != 0 { j += 1 }
                            if dat & 0x80 != 0 { j += 2 }
                        }
                    }
                }
    
                self.np_patoff = j as i16;
            }
        }
    }

    fn getnote(&mut self, module: &S3mData) -> usize {
        if self.np_patseg == 0 /*|| self.np_patseg >= self.mseg_len*/ || self.np_pat >= module.pat_num as i16 {
            return 255
        }

        let mut ch = 255_usize;
        let mut dat = 0_u8;
        let mut i = self.np_patoff as usize;
        let pat = &module.patterns[self.np_pat as usize];
        loop {
            dat = pat.data[i]; i += 1;
            if dat == 0 {  // end of row
                self.np_patoff = i as i16;
                return 255;
            }
    
            if module.ch_settings[dat as usize & 0x1F] & 0x80 == 0 {
                ch = dat as usize & 0x1F;  // channel to trigger
                break
            }
    
            // channel is off, skip data
            if dat & 0x20 != 0 { i += 2 }
            if dat & 0x40 != 0 { i += 1 }
            if dat & 0x80 != 0 { i += 2 }
        }
    
        if dat & 0x20 != 0 {
            self.chn[ch].note = pat.data[i]; i += 1;
            self.chn[ch].ins  = pat.data[i]; i += 1;
    
            if self.chn[ch].note != 255 { self.chn[ch].lastnote = self.chn[ch].note }
            if self.chn[ch].ins  != 0   { self.chn[ch].lastins  = self.chn[ch].ins  }
        }
    
        if dat & 0x40 != 0 {
             self.chn[ch].vol = pat.data[i]; i += 1;
        }
    
        if dat & 0x80 != 0 {
            self.chn[ch].cmd  = pat.data[i]; i += 1;
            self.chn[ch].info = pat.data[i]; i += 1;
        }
    
        self.np_patoff = i as i16;

        ch
    }

    fn doamiga(&mut self, ch: usize, module: &S3mData, mut mixer: &mut Mixer) {
        //uint8_t *insdat;
        //int8_t loop;
        //uint32_t insoffs;
        //uint32_t inslen;
        //uint32_t insrepbeg;
        //uint32_t insrepend;
    
        let note = self.chn[ch].note;
        let ins = self.chn[ch].ins as usize;
        let vol = self.chn[ch].vol;
        let cmd = self.chn[ch].cmd;
        let info = self.chn[ch].info;

        if ins != 0 {
            self.chn[ch].lastins = ins as u8;
            self.chn[ch].astartoffset = 0;
    
            if ins <= module.ins_num as usize {  // added for safety reasons
                let insdat = &module.instruments[ins];
                if insdat.typ != 0 {
                    if insdat.typ == 1 {
                        self.chn[ch].ac2spd = insdat.c2spd as i32;
    
                        if self.tracker == OPENMPT || self.tracker == BEROTRACKER {
                            if cmd == ('S' as u8 - 64) && info & 0xF0 == 0x20 {
                                self.chn[ch].ac2spd = XFINETUNE_AMIGA[info as usize & 0x0F] as i32;
                            }
                        }
    
                        self.chn[ch].avol = insdat.vol;

                        if self.chn[ch].avol < 0 {
                            self.chn[ch].avol = 0;
                        } else if self.chn[ch].avol > 63 {
                            self.chn[ch].avol = 63;
                        }

                        self.chn[ch].aorgvol = self.chn[ch].avol;
                        self.setvol(ch, &mut mixer);

                        mixer.set_patch(ch, ins - 1, ins - 1);
    
/*
                        insoffs = ((insdat[0x0D] << 16) | (insdat[0x0F] << 8) | insdat[0x0E]) * 16;
    
                        inslen    = *((uint32_t *)(&insdat[0x10]));
                        insrepbeg = *((uint32_t *)(&insdat[0x14]));
                        insrepend = *((uint32_t *)(&insdat[0x18]));
    
                        if (insrepbeg > inslen) insrepbeg = inslen;
                        if (insrepend > inslen) insrepend = inslen;
    
                        loop = 0;
                        if ((insdat[0x1F] & 1) && inslen && (insrepend > insrepbeg))
                            loop = 1;
    
                        // This specific portion differs from what sound card driver you use in ST3...
                        if self.soundcardtype == Soundcard::Sb || cmd != ('G' as u8 - 64) && .cmd != ('L' as u8 - 64) {
                            self.voice_set_source(ch, (const int8_t *)(&mseg[insoffs]), inslen,
                                insrepend - insrepbeg, insrepend, loop,
                                insdat[0x1F] & 4, insdat[0x1F] & 2);
                        }
*/
                    } else {
                        self.chn[ch].lastins = 0;
                    }
                }
            }
        }
    
        // continue only if we have an active instrument on this channel
        if self.chn[ch].lastins == 0 {
             return
        }
    
        if cmd == ('O' as u8 - 64) {
            if info == 0 {
                self.chn[ch].astartoffset = self.chn[ch].astartoffset00;
            } else {
                self.chn[ch].astartoffset   = 256 * info as u16;
                self.chn[ch].astartoffset00 = self.chn[ch].astartoffset;
            }
        }
    
        if note != 255 {
            if note == 254 {
                self.chn[ch].aspd    = 0;
                self.chn[ch].avol    = 0;
                self.chn[ch].asldspd = 65535;
    
                self.setspd(ch, &mut mixer);
                self.setvol(ch, &mut mixer);
    
                // shutdown channel
                //self.voice_set_source(ch, NULL, 0, 0, 0, 0, 0, 0);
                mixer.set_voicepos(ch, 0.0);
            } else {
                self.chn[ch].lastnote = note;
    
                if cmd != ('G' as u8 - 64) && cmd != ('L' as u8 - 64) {
                    mixer.set_voicepos(ch, self.chn[ch].astartoffset as f64);
                }
    
/*
                if tracker == OPENMPT || tracker == BEROTRACKER {
                    voiceSetPlayBackwards(ch, 0);
                    if ((chn[ch].cmd == ('S' - 64)) && (chn[ch].info == 0x9F))
                        voiceSetReadPosToEnd(ch);
                }
*/
    
                if self.chn[ch].aorgspd == 0 || (cmd != ('G' as u8 - 64) && cmd != ('L' as u8 - 64)) {
                    let h = self.stnote2herz(note) as i32;
                    self.chn[ch].aspd    = self.scalec2spd(ch, h);
                    self.chn[ch].aorgspd = self.chn[ch].aspd;
                    self.chn[ch].avibcnt = 0;
                    self.chn[ch].apancnt = 0;
    
                    self.setspd(ch, &mut mixer);
                }
    
                let h = self.stnote2herz(note) as i32;
                self.chn[ch].asldspd = self.scalec2spd(ch, h);
            }
        }
    
        if vol != 255 {
            if vol <= 64 {
                self.chn[ch].avol    = vol as i8;
                self.chn[ch].aorgvol = vol as i8;
    
                self.setvol(ch, &mut mixer);
    
                return;
            }
    
/*
            /* NON-ST3, but let's handle it no matter what tracker */
            if ((chn[ch].vol >= 128) && (chn[ch].vol <= 192))
            {
                chn[ch].surround = 0;
                voiceSetSurround(ch, 0);
    
                chn[ch].apanpos = (chn[ch].vol - 128) * 4;
                setpan(ch);
            }
*/
        }
    }
    
    fn donewnote(&mut self, ch: usize, notedelayflag: bool, module: &S3mData, mut mixer: &mut Mixer) {
        if notedelayflag {
            self.chn[ch].achannelused = 0x81;
        } else {
            if self.chn[ch].channelnum > self.lastachannelused {
                self.lastachannelused = self.chn[ch].channelnum + 1;
    
                // sanity fix
                if self.lastachannelused > 31 {
                    self.lastachannelused = 31;
                }
            }
    
            self.chn[ch].achannelused = 0x01;
    
            if self.chn[ch].cmd == ('S' as u8 - 64) {
                if (self.chn[ch].info & 0xF0) == 0xD0 {
                    return
                }
            }
        }
    
        self.doamiga(ch, &module, &mut mixer);
    }
    
    fn donotes(&mut self, module: &S3mData, mut mixer: &mut Mixer) {
        for i in 0..32 {
            self.chn[i].note = 255;
            self.chn[i].vol  = 255;
            self.chn[i].ins  = 0;
            self.chn[i].cmd  = 0;
            self.chn[i].info = 0;
        }
    
        self.seekpat(&module);
    
        loop {
            let ch = self.getnote(&module);
            if ch == 255 {
                break  // end of row/channels
            }
    
            if module.ch_settings[ch] & 0x7F <= 15 {  // no adlib channel types yet
                self.donewnote(ch, false, &module, &mut mixer);
            }
        }
    }
    
    fn docmd1(&mut self, module: &S3mData, mixer: &mut Mixer) {
    }

    fn docmd2(&mut self, module: &S3mData, mixer: &mut Mixer) {
    }

    // periodically called from mixer
    fn dorow(&mut self, module: &S3mData, mut mixer: &mut Mixer) {
        self.patmusicrand = ((((self.patmusicrand as u32 * 0xCDEF) >> 16) as u16).wrapping_add(0x1727)) & 0x0000FFFF;
    
        if self.musiccount == 0 {
            if self.patterndelay != 0 {
                self.np_row -= 1;
                self.docmd1(&module, &mut mixer);
                self.patterndelay -= 1;
            } else {
                self.donotes(&module, &mut mixer);
                self.docmd1(&module, &mut mixer);
            }
        } else {
            self.docmd2(&module, &mut mixer);
        }
    
        self.musiccount += 1;
        if self.musiccount >= self.musicmax + self.tickdelay as u8 {
            self.tickdelay = 0;
            self.np_row += 1;
    
            if self.jumptorow != -1 {
                self.np_row = self.jumptorow;
                self.jumptorow = -1;
            }
    
            // np_row = 0..63, 64 = get new pat
            if self.np_row >= 64 || (self.patloopcount == 0 && self.breakpat != 0) {
                if self.breakpat == 255 {
                    self.breakpat = 0;
                    //self.Playing  = 0;
    
                    return;
                }
    
                self.breakpat = 0;
                self.np_row = self.neworder(&module);  // if breakpat, np_row = break row
            }
    
            self.musiccount = 0;
        }
    }

    fn loadheaderparms(&mut self, module: &S3mData) {
        //uint8_t *insdat;
        //uint16_t insnum;
        //uint32_t i;
        //uint32_t j;
        //uint32_t inslen;
        //uint32_t insoff;
    
        // set to init defaults first
        self.oldstvib = false;
        self.setspeed(6);
        self.settempo(125);
        self.aspdmin = 64;
        self.aspdmax = 32767;
        self.globalvol = 64;
        self.amigalimits = false;
        self.fastvolslide = false;
        //self.setStereoMode(0);
        //self.setMasterVolume(48);
    
        self.tracker = (module.cwt_v >> 12) as u8;
    
        if module.m_v != 0 {
            if module.m_v & 0x80 != 0 {
                //self.setStereoMode(1);
            }
    
            if module.m_v & 0x7F != 0 {
                if module.m_v & 0x7F < 16 {
                    //setMasterVolume(16);
                } else {
                    //setMasterVolume(mseg[0x33] & 0x7F);
                }
            }
        }
    
        if module.i_t != 0 {
            self.settempo(module.i_t as u16);
        }
    
        if module.i_s != 255 {
            self.setspeed(module.i_s);
        }
    
        if module.g_v != 255 {
            self.globalvol = module.g_v as i16;
            if self.globalvol > 64 {
                self.globalvol = 64;
            }
        }
    
        if module.flags & 0xff != 255 {
            self.amigalimits  = module.flags & 0x10 != 0;
            self.fastvolslide = module.flags & 0x40 != 0;
    
            if self.amigalimits {
                self.aspdmax = 1712 * 2;
                self.aspdmin =  907 / 2;
            }
        }
    
        // force fastvolslide if ST3.00
        if module.cwt_v == 0x1300 {
            self.fastvolslide = true;
        }
    
        self.oldstvib = module.flags & 0x01 != 0;
    
/*
        if module.ffi != 0 {
            // we have unsigned samples, convert to signed
    
            for i in 0..module.ins_num {
                insdat = &mseg[*((uint16_t *)(&mseg[instrumentadd + (i * 2)])) * 16];
                insoff = ((insdat[0x0D] << 16) | (insdat[0x0F] << 8) | insdat[0x0E]) * 16;
    
                if (insoff && (insdat[0] == 1))
                {
                    inslen = *((uint32_t *)(&insdat[0x10]));
    
                    if (insdat[0x1F] & 2) inslen *= 2; /* stereo */
    
                    if (insdat[0x1F] & 4)
                    {
                        /* 16-bit */
                        for (j = 0; j < inslen; ++j)
                            *((int16_t *)(&mseg[insoff + (j * 2)])) = *((uint16_t *)(&mseg[insoff + (j * 2)])) - 0x8000;
                    }
                    else
                    {
                        /* 8-bit */
                        for (j = 0; j < inslen; ++j)
                            mseg[insoff + j] = mseg[insoff + j] - 0x80;
                    }
                }
            }
        }
*/
    }
}

impl FormatPlayer for St3Play {
    fn start(&mut self, data: &mut PlayerData, mdata: &ModuleData, _mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<S3mData>().unwrap();

        self.loadheaderparms(&module);

        for i in 0..32 {
            self.chn[i].channelnum   = i as u8;
            self.chn[i].achannelused = 0x80;
            self.chn[i].chanvol      = 0x40;
        }

        self.lastachannelused = 1;

        data.speed = self.musicmax as usize;
        data.tempo = self.tempo as usize;

        self.np_ord = 0;
        self.neworder(&module);
    }

    fn play(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<S3mData>().unwrap();

        self.dorow(&module, &mut mixer);

        data.frame = self.musiccount as usize;
        data.row = self.np_row as usize;
        data.pos = self.np_ord - 1 as usize;

        data.speed = self.musicmax as usize;
        data.tempo = self.tempo as usize;
    }

    fn reset(&mut self) {
    }
}

