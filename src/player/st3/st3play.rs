use module::Module;
use player::{PlayerData, Virtual, FormatPlayer};

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
    np_patseg         : u32,
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

    //*mseg = NULL : u8,
    //instrumentadd : u16,
    lastachannelused : i8,
    tracker : u8,
    oldstvib : i8,
    fastvolslide : i8,
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

    fn setspd(&mut self, ch: usize) {
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
            self.voice_set_sampling_frequency(ch, 0);
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
            self.voice_set_sampling_frequency(ch, 14317056 / tmpspd as u32);
        }
    }

    fn setvol(&mut self, ch: usize) {
        self.chn[ch].achannelused |= 0x80;
        self.voice_set_volume(ch, (self.chn[ch].avol as f64 / 63.0_f64) * (self.chn[ch].chanvol as f64 / 64.0_f64) *
                                  (self.globalvol as f64 / 64.0_f64), self.chn[ch].apanpos);
    }

    fn setpan(&mut self, ch: usize) {
        self.voice_set_volume(ch, (self.chn[ch].avol as f64 / 63.0_f64) * (self.chn[ch].chanvol as f64 / 64.0_f64) *
                                  (self.globalvol as f64 / 64.0_f64), self.chn[ch].apanpos);
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

    fn neworder(&mut self, module: &Module) -> i16 {
        loop {
            self.np_ord += 1;

            if module.orders.pattern(self.np_ord - 1) == 255 || self.np_ord > module.orders.num(0) {  // end
                self.np_ord = 1;
            }

            if module.orders.pattern(self.np_ord - 1) == 254 {  // skip
                continue;  // goto newOrderSkip;
            }

            self.np_pat       = module.orders.pattern(self.np_ord - 1) as i16;
            self.np_patoff    = -1;  // force reseek
            self.np_row       = self.startrow as i16;
            self.startrow     = 0;
            self.patmusicrand = 0;
            self.patloopstart = -1;
            self.jumptorow    = -1;

            return self.np_row;
        }
    }


    fn voice_set_volume(&self, voiceNumber: usize, vol: f64, pan: i16) {
    }

    fn voice_set_sampling_frequency(&self, voiceNumber: usize, samplingFrequency: u32) {
    }
}

impl FormatPlayer for St3Play {
    fn start(&mut self, _data: &mut PlayerData, module: &Module) {
    }

    fn play(&mut self, data: &mut PlayerData, module: &Module, virt: &mut Virtual) {
    }

    fn reset(&mut self) {
    }
}

