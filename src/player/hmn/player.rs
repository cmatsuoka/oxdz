use module::{Module, ModuleData};
use player::{Options, PlayerData, FormatPlayer, State};
use player::scan::SaveRestore;
use format::mk::ModData;
use mixer::Mixer;

/// His Master's Noise Replayer
///
/// An oxdz player based on Musicdisktrackerreplay Pex "Mahoney" Tufvesson
/// (December 1990).
///
/// ## Notes:
///
/// From http://www.livet.se/mahoney/:
///
/// Most modules from His Master's Noise uses special chip-sounds or
/// fine-tuning of samples that never was a part of the standard NoiseTracker
/// v2.0 command set. So if you want to listen to them correctly use an Amiga
/// emulator and run the demo! DeliPlayer does a good job of playing them
/// (there are some occasional error mostly concerning vibrato and portamento
/// effects, but I can live with that!), and it can be downloaded from
/// http://www.deliplayer.com
///
/// ---
///
/// From http://www.cactus.jawnet.pl/attitude/index.php?action=readtext&issue=12&which=12
///
/// [Bepp] For your final Amiga release, the music disk His Master's Noise,
/// you developed a special version of NoiseTracker. Could you tell us a
/// little about this project?
///
/// [Mahoney] I wanted to make a music disk with loads of songs, without being
/// too repetitive or boring. So all of my "experimental features" that did not
/// belong to NoiseTracker v2.0 were put into a separate version that would
/// feature wavetable sounds, chord calculations, off-line filter calculations,
/// mixing, reversing, sample accurate delays, resampling, fades - calculations
/// that would be done on a standard setup of sounds instead of on individual
/// modules. This "compression technique" lead to some 100 songs fitting on two
/// standard 3.5" disks, written by 22 different composers. I'd say that writing
/// a music program does give you loads of talented friends - you should try
/// that yourself someday!
///
/// ---
///
/// From: Pex Tufvesson
/// To: Claudio Matsuoka
/// Date: Sat, Jun 1, 2013 at 4:16 AM
/// Subject: Re: A question about (very) old stuff
///
/// (...)
/// If I remember correctly, these chip sounds were done with several short
/// waveforms, and an index table that was loopable that would choose which
/// waveform to play each frame. And, you didn't have to "draw" every
/// waveform in the instrument - you would choose which waveforms to draw
/// and the replayer would (at startup) interpolate the waveforms that you
/// didn't draw.
///
/// In the special noisetracker, you could draw all of these waveforms, draw
/// the index table, and the instrument would be stored in one of the
/// "patterns" of the song.

#[derive(SaveRestore)]
pub struct HmnPlayer {
    options: Options,

    //l658_instr     : u16,
    l695_counter   : u8,
    l642_speed     : u8,
    l693_songpos   : u8,
    l692_pattpos   : u8,
    //l701_dmacon    : u16,
    //l49_2_vol      : [u16; 4],
    l698_samplestarts: [u32; 31],
    l681_break     : bool,
    voice          : [ChannelData; 4],
}

impl HmnPlayer {
    pub fn new(_module: &Module, options: Options) -> Self {
        HmnPlayer {
            options,

            l642_speed       : 6,
            l693_songpos     : 0,
            l692_pattpos     : 0,
            l695_counter     : 0,
            l681_break       : false,
            l698_samplestarts: [0; 31],
            voice            : [ChannelData::new(); 4],
        }
    }

    fn l505_2_music(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        self.l695_counter += 1;
        if self.l642_speed > self.l695_counter {
            // L505_4_nonew
            for chn in 0..4 {
                self.l577_2_checkcom(chn, &mut mixer);
                self.prog_handler(chn, &module, &mut mixer);
                let ch = &mut self.voice[chn];
                mixer.set_loop_start(chn, ch.n_a_loopstart);
                mixer.set_loop_end(chn, ch.n_a_loopstart + ch.n_e_replen as u32 * 2);
            }
            return
        }

        self.l695_counter = 0;                    // CLR.L   L695
        self.l505_f_getnew(&module, &mut mixer);  // BRA     L505
    }

    fn l505_8_arpeggio(&mut self, chn: usize, mut mixer: &mut Mixer) {
        let val = match self.l695_counter % 3 {
            2 => self.voice[chn].n_3_cmdlo & 15,  // L505_A
            0 => 0,                  // L505_B
            _ => self.voice[chn].n_3_cmdlo >> 4,
        } as usize;

        // L505_C
        for i in 0..36 {
            if self.voice[chn].n_10_period&0xfff >= PERIODS[i] {
                if i+val < PERIODS.len() {  // oxdz: add sanity check
                    // L505_E
                    self.percalc(chn, PERIODS[i+val], &mut mixer);
                    return
                }
            }
        }
    }

    fn l505_f_getnew(&mut self, module: &ModData, mut mixer: &mut Mixer) {
        let pat = match module.pattern_in_position(self.l693_songpos as usize) {
            Some(val) => val,
            None      => return,
        };

        for chn in 0..4 {
            self.s505_j_playvoice(pat, chn, &module, &mut mixer);
        }

        // L505_M_setdma
        for chn in 0..4 {
            self.prog_handler(chn, &module, &mut mixer);
            let ch = &mut self.voice[chn];
            mixer.set_loop_start(chn, ch.n_a_loopstart);
            mixer.set_loop_end(chn, ch.n_a_loopstart + ch.n_e_replen as u32 * 2);
        }

        // L505_R
        // L505_HA
        self.l692_pattpos +=1;
        loop {
            if self.l692_pattpos == 64 {
                // HAHA
                self.l692_pattpos = 0;                                  // CLR.L   L692
                self.l681_break = false;                                // CLR.W   L681
                self.l693_songpos = self.l693_songpos.wrapping_add(1);  // ADDQ.L  #1,L693
                self.l693_songpos &= 0x7f;                              // AND.L   #$7F,L693
                if self.l693_songpos >= module.song_length {
                    self.l693_songpos = module.restart;                 // MOVE.B  $3B7(A0),D0 / MOVE.L  D0,L693
                }
            }
            // L505_U
            if self.l681_break {            // TST.W   L681
                self.l692_pattpos = 64;
                continue                    // BNE.S   HAHA
            }
            break
        }
    }

    fn s505_j_playvoice(&mut self, pat: usize, chn: usize, module: &ModData, mut mixer: &mut Mixer) {
        let event = module.patterns.event(pat, self.l692_pattpos, chn);
        { 
            let ch = &mut self.voice[chn];
    
            ch.n_0_note = event.note;       // MOVE.L  (A0,D1.L),(A6)
            ch.n_2_cmd = event.cmd;
            ch.n_3_cmdlo = event.cmdlo;
    
            let ins = (((event.note & 0xf000) >> 8) | ((event.cmd as u16 & 0xf0) >> 4)) as usize;
    
            if ins > 0 && ins <= 31 {  // sanity check added: was: ins != 0
                let prog_ins = ins as usize - 1;
                let instrument = &module.instruments[prog_ins];
                ch.n_4_samplestart = self.l698_samplestarts[prog_ins];         // MOVE.L  $0(A1,D2.L),$04(A6)     ;instrmemstart
                ch.n_1e_finetune = instrument.finetune;                        // MOVE.B  -$16+$18(A3,D4.L),$1E(A6)       ;CURRSTAMM
                ch.n_1c_prog_on = false;                                       // clr.b   $1c(a6) ;prog on/off
                if &instrument.name[..4] == "Mupp" {                           // CMP.L   #'Mupp',-$16(a3,d4.l)
                    let insname = instrument.name.as_bytes();
                    ch.n_1c_prog_on = true;                                    // move.b  #1,$1c(a6)      ;prog on
                    ch.n_4_samplestart = 0x43c + 0x400 * insname[4] as u32;    // proginstr data-start
                    ch.prog_ins = prog_ins;
                    // we loaded pattern data as sample data
                    let sample = &module.samples()[prog_ins];
                    ch.n_12_volume = sample.data[0x3c0] & 0x7f;                // MOVE.B  $3C0(A0),$12(A6) / AND.B   #$7F,$12(A6)
                    ch.n_a_loopstart = 32 * sample.data[0x380] as u32;         // loopstartmempoi = startmempoi
                    ch.n_13_volume = instrument.volume;                        // move.B  $3(a3,d4.l),$13(a6)     ;volume
                    ch.n_8_dataloopstart = insname[5];                         // move.b  -$16+$5(a3,d4.l),8(a6)  ;dataloopstart
                    ch.n_9_dataloopend = insname[6];                           // move.b  -$16+$6(a3,d4.l),9(a6)  ;dataloopend
                    ch.n_8_length = ((insname[5] as u16) << 8) | insname[6] as u16;  // ouch! that was a nasty variable reuse trick
                    ch.n_e_replen = 0x10;                                      // move.w  #$10,$e(a6)     ;looplen
                } else {
                    // noprgo
                    ch.n_8_length = instrument.size;                           // MOVE.W  $0(A3,D4.L),$08(A6)
                    ch.n_13_volume = instrument.volume as u8;                  // MOVE.W  $2(A3,D4.L),$12(A6)
                    ch.n_12_volume = 0x40;                                     // move.b  #$40,$12(a6)
                    if instrument.repeat != 0 {                                // MOVE.W  $4(A3,D4.L),D3 / TST.W   D3
                        ch.n_a_loopstart = instrument.repeat as u32;           // MOVE.L  D2,$A(A6)       ;LOOPSTARTPOI
                        ch.n_8_length = instrument.repeat + instrument.replen; // MOVE.W  $4(A3,D4.L),D0  ;REPEAT
                                                                               // ADD.W   $6(A3,D4.L),D0  ;+REPLEN
                                                                               // MOVE.W  D0,$8(A6)       ;STARTLENGTH
                        ch.n_e_replen = instrument.replen;                     // MOVE.W  $6(A3,D4.L),$E(A6);LOOPLENGTH
                    } else {
                        // L505_K_noloop
                        ch.n_e_replen = instrument.replen;
                    }
                }
                // L505_LQ
                //let volume = (ch.n_13_volume as u16 * ch.n_12_volume as u16) >> 6;
                //mixer.set_volume(chn, volume << 4);
            }
        }

        // L505_L_setregs
        if self.voice[chn].n_0_note & 0xfff != 0 {
            if self.voice[chn].n_8_length != 0 {        // TST.W  8(A6) / BEQ.S  STOPSOUNDET
                match self.voice[chn].n_2_cmd & 0xf {
                    0x05 => {  // MYPI
                        self.setmyport(chn);
                        self.l577_2_checkcom2(chn, &mut mixer)
                    },
                    0x03 => {
                        self.setmyport(chn);
                        self.l577_2_checkcom2(chn, &mut mixer)
                    },
                    _    => {  // JUP
                        {
                            let ch = &mut self.voice[chn];
                            ch.n_10_period = (ch.n_0_note & 0xfff) as i16;
                            ch.n_1b_vibpos = 0;                    // CLR.B   $1B(A6)
                            ch.n_1d_prog_datacou = 0;              // clr.b   $1d(a6) ;proglj-datacou
                            if ch.n_1c_prog_on {
                                mixer.set_sample_ptr(chn, ch.n_4_samplestart);
                                mixer.set_loop_start(chn, ch.n_a_loopstart);
                                mixer.set_loop_end(chn, ch.n_a_loopstart + ch.n_e_replen as u32 * 2);
                            } else {
                                // normalljudstart
                                mixer.set_sample_ptr(chn, ch.n_4_samplestart);
                                mixer.set_loop_start(chn, ch.n_a_loopstart);
                                //mixer.set_loop_end(chn, ch.n_8_length as u32);
                                mixer.set_loop_end(chn, ch.n_a_loopstart + ch.n_e_replen as u32 * 2);
                            }
                            mixer.enable_loop(chn, ch.n_e_replen > 1);
                        }
                        // onormalljudstart
                        let period = self.voice[chn].n_10_period & 0xfff;
                        self.percalc(chn, period, &mut mixer);
                    },
                }
            }
        }
        // STOPSOUNDET
        // EFTERSTOPSUND
        // L505_L_setregs2
        self.l577_2_checkcom2(chn, &mut mixer);
    }

    fn prog_handler(&mut self, chn: usize, module: &ModData, mixer: &mut Mixer) {
        let ch = &mut self.voice[chn];

        if ch.n_1c_prog_on {
            let mut datacou = ch.n_1d_prog_datacou;
            let sample = &module.samples()[ch.prog_ins];
            let index = 0x380 + datacou as usize;
            ch.n_12_volume = sample.data[index + 0x40] & 0x7f;     // progvolume
            ch.n_a_loopstart = sample.data[index] as u32 * 0x20;   // loopstartmempoi
            datacou += 1;
            if datacou > ch.n_9_dataloopend {
                datacou = ch.n_8_dataloopstart;
            }
            // norestartofdata
            ch.n_1d_prog_datacou = datacou;
        }
        // norvolum
        let volume = (ch.n_12_volume as u16 * ch.n_13_volume as u16) as usize >> 6;
        mixer.set_volume(chn, volume << 4);
    }

    fn setmyport(&mut self, chn: usize) {
        let ch = &mut self.voice[chn];
        ch.n_18_wantperiod = (ch.n_0_note & 0xfff) as i16;
        ch.n_16_portdir = false;            // clr.b   $16(a6)
        if ch.n_10_period == ch.n_18_wantperiod {
            // clrport
            ch.n_18_wantperiod = 0;         // clr.w   $18(a6)
        } else if ch.n_10_period < ch.n_18_wantperiod {
            ch.n_16_portdir = true;         // move.b  #$1,$16(a6)
        }
    }

    fn myport(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.voice[chn];
            if ch.n_3_cmdlo != 0 {
                ch.n_17_toneportspd = ch.n_3_cmdlo;
                ch.n_3_cmdlo = 0;
            }
        }
        self.myslide(chn, &mut mixer)
    }

    fn myslide(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.voice[chn];

        if ch.n_18_wantperiod != 0 {
            if ch.n_16_portdir {
                ch.n_10_period += ch.n_17_toneportspd as i16;
                if ch.n_10_period > ch.n_18_wantperiod {
                    ch.n_10_period = ch.n_18_wantperiod;
                    ch.n_18_wantperiod = 0;
                }
            } else {
                // MYSUB
                ch.n_10_period -= ch.n_17_toneportspd as i16;
                if ch.n_10_period < ch.n_18_wantperiod {
                    ch.n_10_period = ch.n_18_wantperiod;
                    ch.n_18_wantperiod = 0;
                }
            }
        }
        mixer.set_period(chn, ch.n_10_period as f64);  // move.w  $10(a6),$6(a5)
    }

    fn vib(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.voice[chn];
            if ch.n_3_cmdlo != 0 {
                ch.n_1a_vibrato = ch.n_3_cmdlo;
            }
        }
        self.vibrato(chn, &mut mixer)
    }

    fn vibrato(&mut self, chn: usize, mut mixer: &mut Mixer) {
        let period = {
            let ch = &mut self.voice[chn];
    
            let pos = (ch.n_1b_vibpos >> 2) & 0x1f;
            let val = SIN[pos as usize];
            let amt = ((val as usize * (ch.n_1a_vibrato & 0xf) as usize) >> 7) as i16;
    
            let mut period = ch.n_10_period;
            if ch.n_1b_vibpos & 0x80 == 0 {
                period += amt
            } else {
                // VIBMIN
                period -= amt
            }
            period
        };

        self.percalc(chn, period, &mut mixer);

        let ch = &mut self.voice[chn];
        ch.n_1b_vibpos = ch.n_1b_vibpos.wrapping_add((ch.n_1a_vibrato >> 2) & 0x3c);
    }

    fn megaarp(&mut self, chn: usize, mut mixer: &mut Mixer) {
        let val = {
            let ch = &mut self.voice[chn];
            let pos = ch.n_1b_vibpos;
            ch.n_1b_vibpos = ch.n_1b_vibpos.wrapping_add(1);
            let index = ((ch.n_3_cmdlo & 0xf) << 4) + (pos & 0xf);
            MEGA_ARPS[index as usize] as usize
        };

        // MegaAlo
        for i in 0..36 {
            if self.voice[chn].n_10_period&0xfff >= PERIODS[i] {
                let mut index = i + val;
                while index >= PERIODS.len() {
                    index -= 12;
                }
                // MegaOk
                self.percalc(chn, PERIODS[index], &mut mixer);
                return
            }
        }
    }

    fn percalc(&mut self, chn: usize, val: i16, mixer: &mut Mixer) {
        mixer.set_period(chn, (((self.voice[chn].n_1e_finetune as i8 as i16 * val) >> 8) + val) as f64);
    }


    fn l577_2_checkcom(&mut self, chn: usize, mut mixer: &mut Mixer) {
        let cmd = self.voice[chn].n_2_cmd & 0xf;
        if self.voice[chn].n_2_cmd & 0x0f == 0 && self.voice[chn].n_3_cmdlo == 0 {
            // NEJDU
            let period = self.voice[chn].n_10_period;
            self.percalc(chn, period, &mut mixer);
            return
        }
        match cmd {
            0x0 => self.l505_8_arpeggio(chn, &mut mixer),
            0x1 => self.l577_7_portup(chn, &mut mixer),
            0x2 => self.l577_9_portdown(chn, &mut mixer),
            0x3 => self.myport(chn, &mut mixer),
            0x4 => self.vib(chn, &mut mixer),
            0x5 => self.myportvolslide(chn, &mut mixer),
            0x6 => self.vibvolslide(chn, &mut mixer),
            0x7 => self.megaarp(chn, &mut mixer),
            _   => {
                let period = self.voice[chn].n_10_period;
                self.percalc(chn, period, &mut mixer);
                match cmd {
                    0xa => self.l577_3_volslide(chn),
                    _   => {},
                }
            }
        }
    }

    fn l577_3_volslide(&mut self, chn: usize) {
        let ch = &mut self.voice[chn];
        if ch.n_3_cmdlo >> 4 == 0 {
            // mt_voldown
            let cmdlo = ch.n_3_cmdlo & 0x0f;
            if ch.n_13_volume > cmdlo {
                ch.n_13_volume -= cmdlo;
            } else {
                ch.n_13_volume = 0;
            }
        } else {
            ch.n_13_volume += ch.n_3_cmdlo >> 4;
            if ch.n_13_volume > 0x40 {
                ch.n_13_volume = 0x40;
            }
        }
    }

    fn vibvolslide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        self.vibrato(chn, &mut mixer);
        self.l577_3_volslide(chn);
    }

    fn myportvolslide(&mut self, chn: usize, mut mixer: &mut Mixer) {
        self.myslide(chn, &mut mixer);
        self.l577_3_volslide(chn);
    }

    fn l577_7_portup(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.voice[chn];
            ch.n_10_period -= ch.n_3_cmdlo as i16;
            if (ch.n_10_period & 0xfff) < 0x71 {
                ch.n_10_period = (ch.n_10_period as u16 & 0xf000) as i16;
                ch.n_10_period |= 0x71;
            }
        }
        // L577_8
        let period = self.voice[chn].n_10_period;
        self.percalc(chn, period, &mut mixer)
    }

    fn l577_9_portdown(&mut self, chn: usize, mut mixer: &mut Mixer) {
        {
            let ch = &mut self.voice[chn];
            ch.n_10_period += ch.n_3_cmdlo as i16;
            if (ch.n_10_period & 0xfff) >= 0x358 {
                ch.n_10_period = (ch.n_10_period as u16 & 0xf000) as i16;
                ch.n_10_period |= 0x358;
            }
        }
        // L577_A
        let period = self.voice[chn].n_10_period;
        self.percalc(chn, period, &mut mixer)
    }

    fn l577_2_checkcom2(&mut self, chn: usize, mut mixer: &mut Mixer) {
        match self.voice[chn].n_2_cmd & 0xf {
            0xe => self.l577_h_setfilt(chn, &mut mixer),
            0xd => self.l577_i_pattbreak(),
            0xb => self.l577_j_mt_posjmp(chn),
            0xc => self.l577_k_setvol(chn),
            0xf => self.l577_m_setspeed(chn),
            _   => {
                self.l577_2_checkcom(chn, &mut mixer)
            },
        }
    }

    fn l577_h_setfilt(&mut self, chn: usize, mixer: &mut Mixer) {
        let ch = &mut self.voice[chn];
        mixer.enable_filter(ch.n_3_cmdlo & 0x0f != 0);
    }

    fn l577_i_pattbreak(&mut self) {
        self.l681_break = !self.l681_break;
    }

    fn l577_j_mt_posjmp(&mut self, chn: usize) {
        let ch = &mut self.voice[chn];
        self.l693_songpos = ch.n_3_cmdlo.wrapping_sub(1);
        self.l681_break = !self.l681_break;
    }

    fn l577_k_setvol(&mut self, chn: usize) {
        let ch = &mut self.voice[chn];
        if ch.n_3_cmdlo > 0x40 {            // cmp.b   #$40,$3(a6)
            ch.n_3_cmdlo = 40               // move.b  #$40,$3(a6)
        }
        ch.n_13_volume = ch.n_3_cmdlo;
    }

    fn l577_m_setspeed(&mut self, chn: usize) {
        let ch = &mut self.voice[chn];
        if ch.n_3_cmdlo > 0x1f {            // cmp.b   #$1f,$3(a6)
            ch.n_3_cmdlo = 0x1f;            // move.b  #$1f,$3(a6)
        }
        // mt_sets
        if ch.n_3_cmdlo != 0 {
            self.l642_speed = ch.n_3_cmdlo;   // move.b  d0,l642_speed
            self.l695_counter = 0;            // clr.b   l695_counter
        }
    }
}


#[derive(Clone,Copy,Default)]
struct ChannelData {
    n_0_note         : u16,
    n_2_cmd          : u8,
    n_3_cmdlo        : u8,
    n_4_samplestart  : u32,
    n_8_length       : u16,
    n_a_loopstart    : u32,
    n_e_replen       : u16,
    n_10_period      : i16,
    n_12_volume      : u8,
    //n_14_dma_control: u16,
    n_16_portdir     : bool,
    n_17_toneportspd : u8,
    n_18_wantperiod  : i16,
    n_1a_vibrato     : u8,
    n_1b_vibpos      : u8,
    n_1c_prog_on     : bool,

    // progdata
    n_8_dataloopstart: u8,
    n_9_dataloopend  : u8,
    n_13_volume      : u8,
    n_1d_prog_datacou: u8,
    n_1e_finetune    : u8,

    prog_ins: usize,
}

impl ChannelData {
    pub fn new() -> Self {
        Default::default()
    }
}

static SIN: [u8; 32] = [
    0x00, 0x18, 0x31, 0x4a, 0x61, 0x78, 0x8d, 0xa1, 0xb4, 0xc5, 0xd4, 0xe0, 0xeb, 0xf4, 0xfa, 0xfd,
    0xff, 0xfd, 0xfa, 0xf4, 0xeb, 0xe0, 0xd4, 0xc5, 0xb4, 0xa1, 0x8d, 0x78, 0x61, 0x4a, 0x31, 0x18
];

static MEGA_ARPS: [u8; 256] = [
         0,  3,  7, 12, 15, 12,  7,  3,  0,  3,  7, 12, 15, 12,  7,  3,
         0,  4,  7, 12, 16, 12,  7,  4,  0,  4,  7, 12, 16, 12,  7,  4,
         0,  3,  8, 12, 15, 12,  8,  3,  0,  3,  8, 12, 15, 12,  8,  3,
         0,  4,  8, 12, 16, 12,  8,  4,  0,  4,  8, 12, 16, 12,  8,  4,
         0,  5,  8, 12, 17, 12,  8,  5,  0,  5,  8, 12, 17, 12,  8,  5,
         0,  5,  9, 12, 17, 12,  9,  5,  0,  5,  9, 12, 17, 12,  9,  5,
        12,  0,  7,  0,  3,  0,  7,  0, 12,  0,  7,  0,  3,  0,  7,  0,
        12,  0,  7,  0,  4,  0,  7,  0, 12,  0,  7,  0,  4,  0,  7,  0,

         0,  3,  7,  3,  7, 12,  7, 12, 15, 12,  7, 12,  7,  3,  7,  3,
         0,  4,  7,  4,  7, 12,  7, 12, 16, 12,  7, 12,  7,  4,  7,  4,
        31, 27, 24, 19, 15, 12,  7,  3,  0,  3,  7, 12, 15, 19, 24, 27,
        31, 28, 24, 19, 16, 12,  7,  4,  0,  4,  7, 12, 16, 19, 24, 28,
         0, 12,  0, 12,  0, 12,  0, 12,  0, 12,  0, 12,  0, 12,  0, 12,
         0, 12, 24, 12,  0, 12, 24, 12,  0, 12, 24, 12,  0, 12, 24, 12,
         0,  3,  0,  3,  0,  3,  0,  3,  0,  3,  0,  3,  0,  3,  0,  3,
         0,  4,  0,  4,  0,  4,  0,  4,  0,  4,  0,  4,  0,  4,  0,  4
];

static PERIODS: [i16; 38] = [
    0x0358, 0x0328, 0x02fa, 0x02d0, 0x02a6, 0x0280, 0x025c, 0x023a, 0x021a, 0x01fc, 0x01e0,
    0x01c5, 0x01ac, 0x0194, 0x017d, 0x0168, 0x0153, 0x0140, 0x012e, 0x011d, 0x010d, 0x00fe,
    0x00f0, 0x00e2, 0x00d6, 0x00ca, 0x00be, 0x00b4, 0x00aa, 0x00a0, 0x0097, 0x008f, 0x0087,
    0x007f, 0x0078, 0x0071, 0x0000, 0x0000
];


impl FormatPlayer for HmnPlayer {
    fn start(&mut self, data: &mut PlayerData, mdata: &ModuleData, mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<ModData>().unwrap();

        for i in 0..31 {
            self.l698_samplestarts[i] = module.samples[i].address;
        }

        data.speed = 6;
        data.tempo = 125;

        let pan = match self.options.option_int("pan") {
            Some(val) => val,
            None      => 70,
        };
        let panl = -128 * pan / 100;
        let panr = 127 * pan / 100;

        mixer.set_pan(0, panl);
        mixer.set_pan(1, panr);
        mixer.set_pan(2, panr);
        mixer.set_pan(3, panl);

        mixer.enable_paula(true);
    }

    fn play(&mut self, data: &mut PlayerData, mdata: &ModuleData, mut mixer: &mut Mixer) {

        let module = mdata.as_any().downcast_ref::<ModData>().unwrap();

        self.l505_2_music(&module, &mut mixer);

        data.frame = self.l695_counter as usize;
        data.row = self.l692_pattpos as usize;
        data.pos = self.l693_songpos as usize;
        data.speed = self.l642_speed as usize;
        data.frame_time = 20.0;
    }

    fn reset(&mut self) {
        self.l642_speed   = 6;
        self.l695_counter = 0;
        self.l693_songpos = 0;
        self.l681_break   = false;
        self.l692_pattpos = 0;
    }

    unsafe fn save_state(&self) -> State {
        self.save()
    }

    unsafe fn restore_state(&mut self, state: &State) {
        self.restore(&state)
    }
}

// Everything is under control,
// but what is that good for?

