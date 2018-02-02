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
///
/// Non-ST3 additions from other trackers (only handled for non ST3 S3Ms):
///
/// - Mixing:
///   * 16-bit sample support
///   * Stereo sample support
///   * 2^31-1 sample length support
///   * Middle-C speeds beyond 65535
///   * Process the last 16 channels as PCM
///   * Process 8 octaves instead of 7
///   * The ability to play samples backwards
///
/// - Effects:
///   * Command S2x        (set middle-C finetune)
///   * Command S5x        (panbrello type)
///   * Command S6x        (tick delay)
///   * Command S9x        (sound control - only S90/S91/S9E/S9F)
///   * Command Mxx        (set channel volume)
///   * Command Nxy        (channel volume slide)
///   * Command Pxy        (panning slide)
///   * Command Txx<0x20   (tempo slide)
///   * Command Wxy        (global volume slide)
///   * Command Xxx        (7+1-bit pan) + XA4 for 'surround'
///   * Command Yxy        (panbrello)
///   * Volume Command Pxx (set 4+1-bit panning)
///
/// - Variables:
///   * Pan changed from 4-bit (0..15) to 8+1-bit (0..256)
///   * Memory variables for the new N/P/T/W/Y effects
///   * Panbrello counter
///   * Panbrello type
///   * Channel volume multiplier
///   * Channel surround flag
///
/// - Other changes:
///   * Added tracker identification to make sure Scream Tracker 3.xx
///     modules are still played exactly like they should. :-)
///

const SOUNDCARD_GUS  : u8 = 0;  // Default to GUS
const SOUNDCARD_SB   : u8 = 1;

// TRACKER ID
const SCREAM_TRACKER : u8 = 1;
const IMAGO_ORPHEUS  : u8 = 2;
const IMPULSE_TRACKER: u8 = 3;
const SCHISM_TRACKER : u8 = 4;
const OPENMPT        : u8 = 5;
const BEROTRACKER    : u8 = 6;
// there is also CREAMTRACKER (7), but let's ignore that for now

// STRUCTS
#[derive(Default)]
struct Chn {
    aorgvol       : i8,
    avol          : i8,
    channelnum    : u8,
    achannelused  : u8,
    aglis         : bool,  // u8,
    atremor       : u8,
    atreon        : bool,  // u8,
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
    np_ord            : i16,
    np_row            : i16,
    np_pat            : i16,
    np_patoff         : i16,
    patloopstart      : i16,
    jumptorow         : i16,
    //patternadd        : u16,
    patmusicrand      : u16,
    aspdmax           : i32,
    aspdmin           : i32,
    np_patseg         : usize,  // u32,
    chn               : [Chn; 32],
    soundcardtype     : u8,
    //soundBufferSize   : i32,
    //audioFreq         : u32,
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
    //numChannels : u8,
    tempo : i16,
    globalvol : i16,
    stereomode : i8,
    mastervol : u8,
    //mseg_len : u32,
} 


// TABLES
static XFINETUNE_AMIGA: [i16; 16] = [
    7895, 7941, 7985, 8046, 8107, 8169, 8232, 8280,
    8363, 8413, 8463, 8529, 8581, 8651, 8723, 8757
];

static RETRIGVOLADD: [i8; 32] = [
    0, -1, -2, -4, -8,-16,  0,  0,
    0,  1,  2,  4,  8, 16,  0,  0,
    0,  0,  0,  0,  0,  0, 10,  8,
    0,  0,  0,  0,  0,  0, 24, 32
];

static NOTESPD: [u16; 12] = [
    1712 * 16, 1616 * 16, 1524 * 16,
    1440 * 16, 1356 * 16, 1280 * 16,
    1208 * 16, 1140 * 16, 1076 * 16,
    1016 * 16,  960 * 16,  907 * 16
];

static VIBSIN: [i16; 64] = [
     0x00, 0x18, 0x31, 0x4A, 0x61, 0x78, 0x8D, 0xA1,
     0xB4, 0xC5, 0xD4, 0xE0, 0xEB, 0xF4, 0xFA, 0xFD,
     0xFF, 0xFD, 0xFA, 0xF4, 0xEB, 0xE0, 0xD4, 0xC5,
     0xB4, 0xA1, 0x8D, 0x78, 0x61, 0x4A, 0x31, 0x18,
     0x00,-0x18,-0x31,-0x4A,-0x61,-0x78,-0x8D,-0xA1,
    -0xB4,-0xC5,-0xD4,-0xE0,-0xEB,-0xF4,-0xFA,-0xFD,
    -0xFF,-0xFD,-0xFA,-0xF4,-0xEB,-0xE0,-0xD4,-0xC5,
    -0xB4,-0xA1,-0x8D,-0x78,-0x61,-0x4A,-0x31,-0x18
];

static VIBSQU: [u8; 64] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];

static VIBRAMP: [i16; 64] = [
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
    pub fn new(_module: &Module) -> Self {
        Default::default()
    }

    fn getlastnfo(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        if ch.info == 0 {
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

    fn setpan(&mut self, ch: usize, mixer: &mut Mixer) {
        mixer.set_volume(ch, ((self.chn[ch].avol as f64 / 63.0) * (self.chn[ch].chanvol as f64 / 64.0) *
                              (self.globalvol as f64 / 64.0) * 1024.0) as usize);
        mixer.set_pan(ch, self.chn[ch].apanpos as isize);
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
        newspd = (self.stnote2herz((octa << 4) | (lastnote & 0x0F) as u8)) as i32 * 8363;

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

            if module.orders[self.np_ord as usize - 1] == 255 || self.np_ord > module.ord_num as i16 {  // end
                self.np_ord = 1;
            }

            if module.orders[self.np_ord as usize - 1] == 254 {  // skip
                continue;  // goto newOrderSkip;
            }

            self.np_pat       = module.orders[self.np_ord as usize - 1] as i16;
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

        let ch;       // = 255
        let mut dat;  // = 0
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

        let note = self.chn[ch].note;
        let ins = self.chn[ch].ins as usize;
        let vol = self.chn[ch].vol;
        let cmd = self.chn[ch].cmd;
        let info = self.chn[ch].info;

        if ins != 0 {
            self.chn[ch].lastins = ins as u8;
            self.chn[ch].astartoffset = 0;

            if ins <= module.ins_num as usize {  // added for safety reasons
                let insdat = &module.instruments[ins - 1];
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

                        //insoffs = ((insdat[0x0D] << 16) | (insdat[0x0F] << 8) | insdat[0x0E]) * 16;

                        let inslen        = insdat.length;
                        let mut insrepbeg = insdat.loop_beg;
                        let mut insrepend = insdat.loop_end;

                        if insrepbeg > inslen { insrepbeg = inslen }
                        if insrepend > inslen { insrepend = inslen }

                        let has_loop = insdat.flags & 1 != 0 && inslen != 0 && insrepend > insrepbeg;

                        // This specific portion differs from what sound card driver you use in ST3...
                        if self.soundcardtype == SOUNDCARD_SB || cmd != ('G' as u8 - 64) && cmd != ('L' as u8 - 64) {
                            /*self.voice_set_source(ch, (const int8_t *)(&mseg[insoffs]), inslen,
                                insrepend - insrepbeg, insrepend, loop,
                                insdat[0x1F] & 4, insdat[0x1F] & 2);*/
                            mixer.set_patch(ch, ins - 1, ins - 1);
                            mixer.set_loop_start(ch, insrepbeg);
                            mixer.set_loop_end(ch, insrepend);
                            mixer.enable_loop(ch, has_loop);
                        }
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
                mixer.set_volume(ch, 0);
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

    fn docmd1(&mut self, mut mixer: &mut Mixer) {
        for i in 0..self.lastachannelused as usize + 1 {
            if self.chn[i].achannelused != 0 {
                if self.chn[i].info != 0 {
                    self.chn[i].alastnfo = self.chn[i].info;
                }

                if self.chn[i].cmd != 0 {
                    self.chn[i].achannelused |= 0x80;

                    if self.chn[i].cmd == 'D' as u8 - 64 {
                        // fix retrig if Dxy
                        self.chn[i].atrigcnt = 0;

                        // fix speed if tone port noncomplete
                        if self.chn[i].aspd != self.chn[i].aorgspd {
                            self.chn[i].aspd = self.chn[i].aorgspd;
                            self.setspd(i, &mut mixer);
                        }
                    } else {
                        if self.chn[i].cmd != 'I' as u8 - 64 {
                            self.chn[i].atremor = 0;
                            self.chn[i].atreon  = true;
                        }

                        if  self.chn[i].cmd != 'H' as u8 - 64 &&
                            self.chn[i].cmd != 'U' as u8 - 64 &&
                            self.chn[i].cmd != 'K' as u8 - 64 &&
                            self.chn[i].cmd != 'R' as u8 - 64
                        {
                            self.chn[i].avibcnt |= 0x80;
                        }

                        // NON-ST3
                        if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
                            if self.chn[i].cmd != 'Y' as u8 - 64 {
                                self.chn[i].apancnt |= 0x80;
                            }
                        }
                    }

                    if self.chn[i].cmd < 27 {
                        self.volslidetype = 0;
                        self.soncejmp(i, &mut mixer);
                    }
                } else {
                    // NON-ST3
                    if self.tracker != SCREAM_TRACKER { 
                        // recalc pans
                        self.setpan(i, &mut mixer);
                        //voiceSetSurround(i, self.chn[i].surround);
                    }

                    // fix retrig if no command
                    self.chn[i].atrigcnt = 0;

                    // fix speed if tone port noncomplete
                    if self.chn[i].aspd != self.chn[i].aorgspd {
                        self.chn[i].aspd  = self.chn[i].aorgspd;
                        self.setspd(i, &mut mixer);
                    }
                }
            }
        }
    }

    fn docmd2(&mut self, module: &S3mData, mut mixer: &mut Mixer) {
        for i in 0..self.lastachannelused as usize + 1 {
            if self.chn[i].achannelused != 0 {
                if self.chn[i].cmd != 0 {
                    self.chn[i].achannelused |= 0x80;

                    if self.chn[i].cmd < 27 {
                        self.volslidetype = 0;
                        self.sotherjmp(i, &module, &mut mixer);
                    }
                }
            }
        }
    }

    // periodically called from mixer
    fn dorow(&mut self, module: &S3mData, mut mixer: &mut Mixer) {
        self.patmusicrand = ((((self.patmusicrand as u32 * 0xCDEF) >> 16) as u16).wrapping_add(0x1727)) & 0x0000FFFF;

        if self.musiccount == 0 {
            if self.patterndelay != 0 {
                self.np_row -= 1;
                self.docmd1(&mut mixer);
                self.patterndelay -= 1;
            } else {
                self.donotes(&module, &mut mixer);
                self.docmd1(&mut mixer);
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


    // EFFECTS

    fn ssoncejmp(&mut self, i: usize, mut mixer: &mut Mixer) {
        match self.chn[i].cmd >> 4 {
            0x1 => self.s_setgliss(i),
            0x2 => self.s_setfinetune(i),
            0x3 => self.s_setvibwave(i),
            0x4 => self.s_settrewave(i),
            0x5 => self.s_setpanwave(i),  // NON-ST3
            0x6 => self.s_tickdelay(i),   // NON-ST3
            0x7 => self.s_ret(),
            0x8 => self.s_setpanpos(i, &mut mixer),
            0x9 => self.s_sndcntrl(i),
            0xa => self.s_ret(),
            0xb => self.s_patloop(i),
            0xc => self.s_notecut(i),
            0xd => self.s_notedelay(i),
            0xe => self.s_patterdelay(i),
            _   => self.s_ret(),
        }
    }

    fn ssotherjmp(&mut self, i: usize, module: &S3mData, mut mixer: &mut Mixer) {
        match self.chn[i].cmd >> 4 {
            0x1 => self.s_ret(),
            0x2 => self.s_ret(),
            0x3 => self.s_ret(),
            0x4 => self.s_ret(),
            0x5 => self.s_ret(),
            0x6 => self.s_ret(),
            0x7 => self.s_ret(),
            0x8 => self.s_ret(),
            0x9 => self.s_ret(),
            0xa => self.s_ret(),
            0xb => self.s_ret(),
            0xc => self.s_notecutb(i),
            0xd => self.s_notedelayb(i, &module, &mut mixer),
            0xe => self.s_ret(),
            _   => self.s_ret(),
        }
    }

    fn soncejmp(&mut self, i: usize, mut mixer: &mut Mixer) {
        match (self.chn[i].cmd + 64) as char {
            'A' => self.s_setspeed(i),
            'B' => self.s_jmpto(i),
            'C' => self.s_break(i),
            'D' => self.s_volslide(i, &mut mixer),
            'E' => self.s_slidedown(i, &mut mixer),
            'F' => self.s_slideup(i, &mut mixer),
            'G' => self.s_ret(),
            'H' => self.s_ret(),
            'I' => self.s_tremor(i, &mut mixer),
            'J' => self.s_arp(i, &mut mixer),
            'K' => self.s_ret(),
            'L' => self.s_ret(),
            'M' => self.s_chanvol(i, &mut mixer),       // NON-ST3
            'N' => self.s_chanvolslide(i, &mut mixer),  // NON-ST3
            'O' => self.s_ret(),
            'P' => self.s_panslide(i, &mut mixer),      // NON-ST3
            'Q' => self.s_retrig(i, &mut mixer),
            'R' => self.s_ret(),
            'S' => self.s_scommand1(i, &mut mixer),
            'T' => self.s_settempo(i),
            'U' => self.s_ret(),
            'V' => self.s_ret(),
            'W' => self.s_globvolslide(i, &mut mixer),  // NON-ST3
            'X' => self.s_setpan(i, &mut mixer),        // NON-ST3
            'Y' => self.s_panbrello(i, &mut mixer),     // NON-ST3
            _   => self.s_ret(),
        }
    }

    fn sotherjmp(&mut self, i: usize, module: &S3mData, mut mixer: &mut Mixer) {
        match (self.chn[i].cmd + 64) as char {
            'A' => self.s_ret(),
            'B' => self.s_ret(),
            'C' => self.s_ret(),
            'D' => self.s_volslide(i, &mut mixer),
            'E' => self.s_slidedown(i, &mut mixer),
            'F' => self.s_slideup(i, &mut mixer),
            'G' => self.s_toneslide(i, &mut mixer),
            'H' => self.s_vibrato(i, &mut mixer),
            'I' => self.s_tremor(i, &mut mixer),
            'J' => self.s_arp(i, &mut mixer),
            'K' => self.s_vibvol(i, &mut mixer),
            'L' => self.s_tonevol(i, &mut mixer),
            'M' => self.s_ret(),
            'N' => self.s_chanvolslide(i, &mut mixer),  // NON-ST3
            'O' => self.s_ret(),
            'P' => self.s_panslide(i, &mut mixer),      // NON-ST3
            'Q' => self.s_retrig(i, &mut mixer),
            'R' => self.s_tremolo(i, &mut mixer),
            'S' => self.s_scommand2(i, &module, &mut mixer),
            'T' => self.s_settempo(i),                  // NON-ST3 (for tempo slides)
            'U' => self.s_finevibrato(i, &mut mixer),
            'V' => self.s_setgvol(i),
            'W' => self.s_globvolslide(i, &mut mixer),  // NON-ST3
            'X' => self.s_ret(),
            'Y' => self.s_panbrello(i, &mut mixer),     // NON-ST3
            _    => self.s_ret(),
        }
    }

    fn s_ret(&mut self) {
    }
    // ----------------

    fn s_setgliss(&mut self, i: usize) {
        self.chn[i].aglis = self.chn[i].info & 0x0F != 0;
    }

    fn s_setfinetune(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        // this function does nothing in ST3 and many other trackers
        if self.tracker == OPENMPT || self.tracker == BEROTRACKER {
            ch.ac2spd = XFINETUNE_AMIGA[ch.info as usize & 0x0F] as i32;
        }
    } 

    fn s_setvibwave(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        ch.avibtretype = (ch.avibtretype & 0xF0) | ((ch.info << 1) & 0x0F);
    }

    fn s_settrewave(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        ch.avibtretype = ((ch.info << 5) & 0xF0) | (ch.avibtretype & 0x0F);
    }

    fn s_setpanwave(&mut self, i: usize) {  // NON-ST3
        let ch = &mut self.chn[i];
        if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
            ch.apantype = ch.info & 0x0F;
        }
    }

    fn s_tickdelay(&mut self, i: usize) {  // NON-ST3
        let ch = &mut self.chn[i];
        if     self.tracker == OPENMPT
            || self.tracker == BEROTRACKER
            || self.tracker == IMPULSE_TRACKER
            || self.tracker == SCHISM_TRACKER
        {
            self.tickdelay += (ch.info & 0x0F) as i8;
        }
    }

    fn s_setpanpos(&mut self, i: usize, mut mixer: &mut Mixer) {
        let num = {
            let ch = &mut self.chn[i];
            ch.surround = 0;
            //voiceSetSurround(ch->channelnum, 0);

            ch.apanpos = (ch.info & 0x0F) as i16 * 17;
            ch.channelnum as usize
        };
        self.setpan(num, &mut mixer);
    }

    fn s_sndcntrl(&mut self, i: usize) {  // NON-ST3
        let ch = &mut self.chn[i];
        if ch.info & 0x0F == 0x00 {
            if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
                ch.surround = 0;
                //voiceSetSurround(ch.channelnum, 0);
            }
        } else if ch.info & 0x0F == 0x01 {
            if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
                ch.surround = 1;
                //voiceSetSurround(ch.channelnum, 1);
            }
        } else if ch.info & 0x0F == 0x0E {
            if self.tracker == OPENMPT || self.tracker == BEROTRACKER {
                //voiceSetPlayBackwards(ch.channelnum, 0);
            }
        } else if ch.info & 0x0F == 0x0F {
            if self.tracker == OPENMPT || self.tracker == BEROTRACKER {
                //voiceSetPlayBackwards(ch.channelnum, 1);
            }
        }
    }

    fn s_patloop(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        if ch.info & 0x0F == 0 {
            self.patloopstart = self.np_row;
            return;
        }

        if self.patloopcount == 0 {
            self.patloopcount = (ch.info & 0x0F) as i8 + 1;

            if self.patloopstart == -1 {
                self.patloopstart = 0;  // default loopstart
            }
        }

        if self.patloopcount > 1 {
            self.patloopcount -= 1;

            self.jumptorow = self.patloopstart;
            self.np_patoff = -1;  // force reseek
        } else {
            self.patloopcount = 0;
            self.patloopstart = self.np_row + 1;
        }
    }

    fn s_notecut(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        ch.anotecutcnt = ch.info & 0x0F;
    }

    fn s_notecutb(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        if ch.anotecutcnt != 0 {
            ch.anotecutcnt -= 1;
            if ch.anotecutcnt == 0 {
                //voiceSetSamplingFrequency(ch.channelnum, 0);  // cut note
            }
        }
    }

    fn s_notedelay(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        ch.anotedelaycnt = ch.info & 0x0F;
    }

    fn s_notedelayb(&mut self, i: usize, module: &S3mData, mut mixer: &mut Mixer) {
        if self.chn[i].anotedelaycnt != 0 {
            self.chn[i].anotedelaycnt -= 1;
            if self.chn[i].anotedelaycnt == 0 {
                let num = self.chn[i].channelnum as usize;
                self.donewnote(num, true, &module, &mut mixer);  // 1 = notedelay end
            }
        }
    }

    fn s_patterdelay(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        if self.patterndelay == 0 {
            self.patterndelay = (ch.info & 0x0F) as i8;
        }
    }

    fn s_setspeed(&mut self, i: usize) {
        let info = self.chn[i].info;
        self.setspeed(info);
    }

    fn s_jmpto(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        if ch.info != 0xFF {
            self.breakpat = 1;
            self.np_ord = ch.info as i16;
        } else {
            self.breakpat = 255;
        }
    }

    fn s_break(&mut self, i: usize) {
        let ch = &mut self.chn[i];
        self.startrow = ((ch.info >> 4) * 10) + (ch.info & 0x0F);
        self.breakpat = 1;
    }

    fn s_volslide(&mut self, i: usize, mut mixer: &mut Mixer) {
        self.getlastnfo(i);

        let num = {
            let ch = &mut self.chn[i];

            let infohi = (ch.info >> 4) as i8;
            let infolo = (ch.info & 0x0F) as i8;

            if infolo == 0x0F {
                if infohi == 0 {
                    ch.avol -= infolo;
                } else if self.musiccount == 0 {
                    ch.avol += infohi;
                }
            } else if infohi == 0x0F {
                if infolo == 0 {
                    ch.avol += infohi;
                } else if self.musiccount == 0 {
                    ch.avol -= infolo;
                }
            } else if self.fastvolslide || self.musiccount != 0 {
                if infolo == 0 {
                    ch.avol += infohi;
                } else {
                    ch.avol -= infolo;
                }
            } else {
                return  // illegal slide
            }

            if ch.avol < 0 {
                ch.avol = 0;
            } else if ch.avol > 63 {
                ch.avol = 63;
            }

            ch.channelnum as usize
        };

        self.setvol(num, &mut mixer);

        match self.volslidetype {
            1 => self.s_vibrato(i, &mut mixer),
            2 => self.s_toneslide(i, &mut mixer),
            _ => ()
        }
    }

    fn s_slidedown(&mut self, i: usize, mut mixer: &mut Mixer) {
        if self.chn[i].aorgspd != 0 {
            self.getlastnfo(i);

            let num = {
                let ch = &mut self.chn[i];

                if self.musiccount != 0 {
                    if ch.info >= 0xE0 {
                        return  // no fine slides here
                    }

                    ch.aspd += ch.info as i32 * 4;
                    if ch.aspd > 32767 {
                        ch.aspd = 32767;
                    }
                } else {
                    if ch.info <= 0xE0 {
                        return;  // only fine slides here
                    }

                    if ch.info <= 0xF0 {
                        ch.aspd += (ch.info & 0x0F) as i32;
                        if ch.aspd > 32767 {
                            ch.aspd = 32767;
                        }
                    } else {
                        ch.aspd += (ch.info & 0x0F) as i32 * 4;
                        if ch.aspd > 32767 {
                            ch.aspd = 32767;
                        }
                    }
                }

                ch.aorgspd = ch.aspd;
                ch.channelnum as usize
            };

            self.setspd(num, &mut mixer);
        }
    }

    fn s_slideup(&mut self, i: usize, mut mixer: &mut Mixer) {
        if self.chn[i].aorgspd != 0 {
            self.getlastnfo(i);

            let num = {
                let ch = &mut self.chn[i];

                if self.musiccount != 0 {
                    if ch.info >= 0xE0 {
                        return  // no fine slides here
                    }

                    ch.aspd -= ch.info as i32 * 4;
                    if ch.aspd < 0 {
                        ch.aspd = 0;
                    }
                } else {
                    if ch.info <= 0xE0 {
                        return  // only fine slides here
                    }

                    if ch.info <= 0xF0 {
                        ch.aspd -= (ch.info & 0x0F) as i32;
                        if ch.aspd < 0 {
                            ch.aspd = 0;
                        }
                    } else {
                        ch.aspd -= (ch.info & 0x0F) as i32 * 4;
                        if ch.aspd < 0 {
                            ch.aspd = 0;
                        }
                    }
                }

                ch.aorgspd = ch.aspd;
                ch.channelnum as usize
            };

            self.setspd(num, &mut mixer);
        }
    }

    fn s_toneslide(&mut self, i: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.chn[i];

            if self.volslidetype == 2 {  // we came from an Lxy (toneslide+volslide)
                ch.info = ch.alasteff1;
            } else {
                if ch.aorgspd == 0 {
                    if ch.asldspd == 0 {
                        return
                    }

                    ch.aorgspd = ch.asldspd;
                    ch.aspd    = ch.asldspd;
                }

                if ch.info == 0 {
                    ch.info = ch.alasteff1;
                } else {
                    ch.alasteff1 = ch.info;
                }
            }
        }

        if self.chn[i].aorgspd != self.chn[i].asldspd {
             let num = {
                let ch = &mut self.chn[i];
                if ch.aorgspd < ch.asldspd {
                    ch.aorgspd += ch.info as i32 * 4;
                    if ch.aorgspd > ch.asldspd {
                        ch.aorgspd = ch.asldspd;
                    }
                } else {
                    ch.aorgspd -= ch.info as i32 * 4;
                    if ch.aorgspd < ch.asldspd {
                        ch.aorgspd = ch.asldspd;
                    }
                }
                ch.channelnum as usize
            };

            let aorgspd = self.chn[i].aorgspd;
            self.chn[i].aspd = if self.chn[i].aglis {
                self.roundspd(num, aorgspd)
            } else {
                aorgspd
            };

            self.setspd(num, &mut mixer);
        }
    }

    fn s_vibrato(&mut self, i: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.chn[i];

            if self.volslidetype == 1 {  // we came from a Kxy (vibrato+volslide)
                ch.info = ch.alasteff;
            } else {
                if ch.info == 0 {
                    ch.info = ch.alasteff;
                }

                if ch.info & 0xF0 == 0 {
                    ch.info = (ch.alasteff & 0xF0) | (ch.info & 0x0F);
                }

                ch.alasteff = ch.info;
            }
        }

        if self.chn[i].aorgspd != 0 {
            let mut cnt = self.chn[i].avibcnt as usize;

            let num = {
                let ch = &mut self.chn[i];

                let vtype   = (ch.avibtretype & 0x0E) >> 1;
                let mut dat = 0_i32;

                // sine
                if vtype == 0 || vtype == 4 {
                    if vtype == 4 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                           cnt = 0;
                        }
                    }

                    dat = VIBSIN[cnt / 2] as i32;
                }

                // ramp
                else if vtype == 1 || vtype == 5 {
                    if vtype == 5 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBRAMP[cnt / 2] as i32;
                }

                // square
                else if vtype == 2 || vtype == 6 {
                    if vtype == 6 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBSQU[cnt / 2] as i32;
                }

                // random
                else if vtype == 3 || vtype == 7 {
                    if vtype == 7 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBSIN[cnt / 2] as i32;
                    cnt += (self.patmusicrand & 0x1E) as usize;
                }

                dat = ((dat * (ch.info & 0x0F) as i32) >> if self.oldstvib { 4 } else { 5 }) + ch.aorgspd;

                ch.aspd = dat;
                ch.channelnum as usize
            };

            self.setspd(num, &mut mixer);

            self.chn[i].avibcnt = ((cnt + ((self.chn[i].info >> 4) * 2) as usize) & 0x7E) as i16;
        }
    }

    fn s_tremor(&mut self, i: usize, mut mixer: &mut Mixer) {
        self.getlastnfo(i);

        if self.chn[i].atremor != 0 {
            self.chn[i].atremor -= 1;
            return;
        }

        let num = self.chn[i].channelnum as usize;

        if self.chn[i].atreon {
            self.chn[i].atreon = false;

            self.chn[i].avol = 0;
            self.setvol(num, &mut mixer);

            self.chn[i].atremor = self.chn[i].info & 0x0F;
        } else {
            self.chn[i].atreon = true;

            self.chn[i].avol = self.chn[i].aorgvol;
            self.setvol(num, &mut mixer);

            self.chn[i].atremor = self.chn[i].info >> 4;
        }
    }

    fn s_arp(&mut self, i: usize, mut mixer: &mut Mixer) {
        let mut octa: u8;
        let mut note: u8;

        self.getlastnfo(i);

        let num = {
            let ch = &mut self.chn[i];

            let tick = self.musiccount % 3;

            let noteadd = match tick {
                1 => ch.info >> 4,
                2 => ch.info & 0x0F,
                _ => 0,
            };

            // check for octave overflow
            octa =  ch.lastnote & 0xF0;
            note = (ch.lastnote & 0x0F) + noteadd;

            while note >= 12 {
                note -= 12;
                octa += 16;
            }
            ch.channelnum as usize
        };

        let spd = self.stnote2herz(octa | note) as i32;
        self.chn[i].aspd = self.scalec2spd(num, spd);
        self.setspd(num, &mut mixer);
    }


    fn s_chanvol(&mut self, i: usize, mut mixer: &mut Mixer) {  // NON-ST3
        if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
            let num = {
                let ch = &mut self.chn[i];

                if ch.info <= 0x40 {
                    ch.chanvol = ch.info as i8;
                }
                ch.channelnum as usize
            };

            self.setvol(num, &mut mixer);
        }
    }

    fn s_chanvolslide(&mut self, i: usize, mut mixer: &mut Mixer) {  // NON-ST3
        if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
            let num = {
                let ch = &mut self.chn[i];

                if ch.info != 0 {
                    ch.nxymem = ch.info;
                } else {
                    ch.info = ch.nxymem;
                }

                let infohi = (ch.nxymem >> 4) as i8;
                let infolo = (ch.nxymem & 0x0F) as i8;

                if infolo == 0x0F {
                    if infohi == 0 {
                        ch.chanvol -= infolo;
                    } else if self.musiccount == 0 {
                        ch.chanvol += infohi;
                    }
                } else if infohi == 0x0F {
                    if infolo == 0 {
                        ch.chanvol += infohi;
                    } else if self.musiccount == 0 {
                        ch.chanvol -= infolo;
                    }
                } else if self.musiccount != 0 {  // don't rely on fastvolslide flag here
                    if infolo == 0 {
                        ch.chanvol += infohi;
                    } else {
                        ch.chanvol -= infolo;
                    }
                } else {
                    return  // illegal slide
                }

                if ch.chanvol < 0 {
                    ch.chanvol =  0;
                } else if ch.chanvol > 64 {
                    ch.chanvol = 64;
                }
                ch.channelnum as usize
            };

            self.setvol(num, &mut mixer);
        }
    }

    fn s_vibvol(&mut self, i: usize, mut mixer: &mut Mixer) {
        self.volslidetype = 1;
        self.s_volslide(i, &mut mixer);
    }

    fn s_tonevol(&mut self, i: usize, mut mixer: &mut Mixer) {
        self.volslidetype = 2;
        self.s_volslide(i, &mut mixer);
    }


    fn s_panslide(&mut self, i: usize, mut mixer: &mut Mixer) {  // NON-ST3
        if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
            let num = {
                let ch = &mut self.chn[i];

                if ch.info != 0 {
                    ch.pxymem = ch.info;
                } else {
                    ch.info = ch.pxymem;
                }

                let infohi = (ch.pxymem >> 4) as i16;
                let infolo = (ch.pxymem & 0x0F) as i16;

                if infolo == 0x0F {
                    if infohi == 0 {
                        ch.apanpos += infolo * 4;
                    } else if self.musiccount == 0 {
                        ch.apanpos -= infohi * 4;
                    }
                } else if infohi == 0x0F {
                    if infolo == 0 {
                        ch.apanpos -= infohi * 4;
                    } else if self.musiccount == 0 {
                        ch.apanpos += infolo * 4;
                    }
                } else if self.musiccount != 0 {  // don't rely on fastvolslide flag here
                    if infolo == 0 {
                        ch.apanpos -= infohi * 4;
                    } else {
                        ch.apanpos += infolo * 4;
                    }
                } else {
                    return  // illegal slide
                }

                if ch.apanpos < 0 {
                    ch.apanpos = 0;
                } else if ch.apanpos > 256 {
                    ch.apanpos = 256;
                }

                ch.channelnum as usize
            };

            self.setpan(num, &mut mixer);
        }
    }

    fn s_retrig(&mut self, i: usize, mut mixer: &mut Mixer) {
        self.getlastnfo(i);

        let num = {
            let ch = &mut self.chn[i];

            let infohi = ch.info >> 4;

            if ch.info & 0x0F == 0 || ch.atrigcnt < (ch.info & 0x0F) {
                ch.atrigcnt += 1;
                return;
            }

            ch.atrigcnt = 0;

            //voiceSetPlayBackwards(ch.channelnum, 0);
            //voiceSetSamplePosition(ch.channelnum, 0);
            mixer.set_voicepos(ch.channelnum as usize, 0.0);

            if RETRIGVOLADD[16 + infohi as usize] == 0 {
                ch.avol += RETRIGVOLADD[infohi as usize];
            } else {
                ch.avol = ((ch.avol as i16 * RETRIGVOLADD[16 + infohi as usize] as i16) / 16) as i8;
            }

            if ch.avol > 63 {
                ch.avol = 63;
            } else if ch.avol < 0 {
                ch.avol = 0;
            }

            ch.channelnum as usize
        };

        self.setvol(num, &mut mixer);

        self.chn[i].atrigcnt += 1;  // probably a mistake? Keep it anyways.
    }

    fn s_tremolo(&mut self, i: usize, mut mixer: &mut Mixer) {
        self.getlastnfo(i);

        {
            let ch = &mut self.chn[i];
            if ch.info & 0xF0 == 0 {
                ch.info = (ch.alastnfo & 0xF0) | (ch.info & 0x0F);
            }

            ch.alastnfo = ch.info;
        }

        if self.chn[i].aorgvol != 0 {
            let mut cnt = self.chn[i].avibcnt as usize;

            let num = {
                let ch = &mut self.chn[i];

                let ttype   = ch.avibtretype >> 5;
                let mut dat = 0_i16;

                // sine
                if ttype == 0 || ttype == 4 {
                    if ttype == 4 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                           cnt = 0;
                        }
                    }

                    dat = VIBSIN[cnt / 2] as i16;
                }

                // ramp
                else if ttype == 1 || ttype == 5 {
                    if ttype == 5 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBRAMP[cnt / 2] as i16;
                }

                // square
                else if ttype == 2 || ttype == 6 {
                    if ttype == 6 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBSQU[cnt / 2] as i16;
                }

                // random
                else if ttype == 3 || ttype == 7 {
                    if ttype == 7 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBSIN[cnt / 2] as i16;
                    cnt += (self.patmusicrand & 0x1E) as usize;
                }

                dat = ((dat * (ch.info & 0x0F) as i16) >> 7) + ch.aorgvol as i16;

                if dat > 63 {
                    dat = 63;
                } else if dat < 0 {
                    dat = 0;
                }

                ch.avol = dat as i8;

                ch.channelnum as usize
            };
            self.setvol(num, &mut mixer);

            self.chn[i].avibcnt = ((cnt + ((self.chn[i].info >> 4) * 2) as usize) & 0x7E) as i16;
        }
    }

    fn s_scommand1(&mut self, i: usize, mut mixer: &mut Mixer) {
        self.getlastnfo(i);
        self.ssoncejmp(i, &mut mixer);
    }

    fn s_scommand2(&mut self, i: usize, module: &S3mData, mut mixer: &mut Mixer) {
        self.getlastnfo(i);
        self.ssotherjmp(i, &module, &mut mixer);
    }


    fn s_settempo(&mut self, i: usize) {
        {
            let ch = &mut self.chn[i];

            if self.musiccount == 0 && ch.info >= 0x20 {
                self.tempo = ch.info as i16;
            }

            // NON-ST3 tempo slide */
            if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
                if self.musiccount == 0 {
                    if ch.info == 0 {
                        ch.info = ch.txxmem;
                    } else {
                        ch.txxmem = ch.info;
                    }
                } else {
                    if ch.info <= 0x0F {
                        self.tempo -= ch.info as i16;
                        if self.tempo < 32 {
                            self.tempo = 32;
                        }
                    } else if ch.info <= 0x1F {
                        self.tempo += ch.info as i16 - 0x10;
                        if self.tempo > 255 {
                            self.tempo = 255;
                        }
                    }
                }
            }
        }
        // ------------------

        let tempo = self.tempo as u16;
        self.settempo(tempo);
    }

    fn s_finevibrato(&mut self, i: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.chn[i];

            if ch.info == 0 {
                ch.info = ch.alasteff;
            }

            if ch.info & 0xF0 == 0 {
                ch.info = (ch.alasteff & 0xF0) | (ch.info & 0x0F);
            }

            ch.alasteff = ch.info;
        }

        if self.chn[i].aorgspd != 0 {
            let mut cnt = self.chn[i].avibcnt as usize;
            {
                let ch = &mut self.chn[i];

                let vtype   = (ch.avibtretype & 0x0E) >> 1;
                let mut dat = 0_i32;

                // sine
                if vtype == 0 || vtype == 4 {
                    if vtype == 4 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBSIN[cnt / 2] as i32;
                }

                // ramp
                else if vtype == 1 || vtype == 5 {
                    if vtype == 5 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBRAMP[cnt / 2] as i32;
                }

                // square
                else if vtype == 2 || vtype == 6 {
                    if vtype == 6 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBSQU[cnt / 2] as i32;
                }

                // random
                else if vtype == 3 || vtype == 7 {
                    if vtype == 7 {
                        cnt &= 0x7F;
                    } else {
                        if cnt & 0x80 != 0 {
                            cnt = 0;
                        }
                    }

                    dat = VIBSIN[cnt / 2] as i32;
                    cnt += (self.patmusicrand & 0x1E) as usize;
                }

                dat = ((dat * (ch.info & 0x0F) as i32) >> if self.oldstvib { 6 } else { 7 }) + ch.aorgspd;

                ch.aspd = dat;
            }
            self.setspd(i, &mut mixer);

            self.chn[i].avibcnt = ((cnt + ((self.chn[i].info >> 4) * 2) as usize) & 0x7E) as i16;
        }
    }

    fn s_setgvol(&mut self, i: usize) {
        let ch = &mut self.chn[i];

        if ch.info <= 64 {
            self.globalvol = ch.info as i16;
        }
    }

    fn s_globvolslide(&mut self, i: usize, mut mixer: &mut Mixer) {  // NON-ST3
        if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
            {
                let ch = &mut self.chn[i];

                if ch.info != 0 {
                    ch.wxymem = ch.info;
                } else {
                    ch.info = ch.wxymem;
                }

                let infohi = (ch.wxymem >> 4) as i16;
                let infolo = (ch.wxymem & 0x0F) as i16;

                if infolo == 0x0F {
                    if infohi == 0 {
                        self.globalvol -= infolo;
                    } else if self.musiccount == 0 {
                        self.globalvol += infohi;
                    }
                } else if infohi == 0x0F {
                    if infolo == 0 {
                        self.globalvol += infohi;
                    } else if self.musiccount == 0 {
                        self.globalvol -= infolo;
                    }
                } else if self.musiccount != 0 {  // don't rely on fastvolslide flag here
                    if infolo == 0 {
                        self.globalvol += infohi;
                    } else {
                        self.globalvol -= infolo;
                    }
                } else {
                    return  // illegal slide
                }

                if self.globalvol < 0 {
                    self.globalvol = 0;
                } else if self.globalvol > 64 {
                    self.globalvol = 64;
                }
            }

            // update all channels now
            for i in 0..self.lastachannelused as usize + 1 {
                self.setvol(i, &mut mixer);
            }
        }
    }

    fn s_setpan(&mut self, i: usize, mut mixer: &mut Mixer) {  // NON-ST3
        //
        // this one should work even in mono mode
        // for newer trackers that exports as ST3
        //
        if self.chn[i].info <= 0x80 {
            self.chn[i].surround = 0;
            //voiceSetSurround(ch.channelnum, 0);

            self.chn[i].apanpos = self.chn[i].info as i16 * 2;
            let num = self.chn[i].channelnum as usize;
            self.setpan(num, &mut mixer);
        } else if self.chn[i].info == 0xA4 {  // surround
            if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
                self.chn[i].surround = 1;
                //voiceSetSurround(ch.channelnum, 1);
            }
        }
    }

    fn s_panbrello(&mut self, i: usize, mixer: &mut Mixer) {  // NON-ST3
        if self.tracker != SCREAM_TRACKER && self.tracker != IMAGO_ORPHEUS {
            let ch = &mut self.chn[i];

            if self.musiccount == 0 {
                if ch.info == 0 {
                    ch.info = ch.alasteff;
                } else {
                    ch.yxymem = ch.info;
                }

                if ch.info & 0xF0 == 0 {
                    ch.info = (ch.yxymem & 0xF0) | (ch.info & 0x0F);
                }

                if ch.info & 0x0F == 0 {
                    ch.info = (ch.info & 0xF0) | (ch.yxymem & 0x0F);
                }
            }

            let mut cnt = ch.apancnt as usize;
            let ptype   = ch.apantype;
            let mut dat = 0_i16;

            // sine
            if ptype == 0 || ptype == 4 {
                if ptype == 4 {
                    cnt &= 0x7F;
                } else {
                    if cnt & 0x80 != 0 {
                        cnt = 0;
                    }
                }

                dat = VIBSIN[cnt / 2] as i16;
            }

            // ramp
            else if ptype == 1 || ptype == 5 {
                if ptype == 5 {
                    cnt &= 0x7F;
                } else {
                    if cnt & 0x80 != 0 {
                        cnt = 0;
                    }
                }

                dat = VIBRAMP[cnt / 2] as i16;
            }

            // square
            else if ptype == 2 || ptype == 6 {
                if ptype == 6 {
                    cnt &= 0x7F;
                } else {
                    if cnt & 0x80 != 0 {
                        cnt = 0;
                    }
                }

                dat = VIBSQU[cnt / 2] as i16;
            }

            // random
            else if ptype == 3 || ptype == 7 {
                if ptype == 7 {
                    cnt &= 0x7F;
                } else {
                    if cnt & 0x80 != 0 {
                       cnt = 0;
                    }
                }

                dat = VIBSIN[cnt / 2] as i16;
                cnt += (self.patmusicrand & 0x1E) as usize;
            }

            dat = ((dat * (ch.info & 0x0F) as i16) >> 4) + ch.apanpos;

            if dat < 0 {
                dat = 0;
            } else if dat > 256 {
                dat = 256;
            }

            // voiceSetVolume(ch.channelnum,
            //       (chn[ch.channelnum].avol    / 63.0f)
            //     * (chn[ch.channelnum].chanvol / 64.0f)
            //     * (globalvol                   / 64.0f), dat);

            mixer.set_pan(ch.channelnum as usize, dat as isize - 128);

            ch.apancnt = ((cnt + ((ch.info >> 6) * 2) as usize) & 0x7E) as i16;
        }
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
        data.pos = self.np_ord as usize - 1;

        data.speed = self.musicmax as usize;
        data.tempo = self.tempo as usize;
    }

    fn reset(&mut self) {
    }
}

