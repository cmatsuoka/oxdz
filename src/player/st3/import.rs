use format::mk::{self, ModData, ModPatterns};
use format::s3m::{S3mData, S3mInstrument, S3mPattern};
use module::Module;
use ::*;

static FINETUNE_TABLE: [u16; 16] = [
    8363, 8413, 8463, 8529, 8581, 8651, 8723, 8757,
    7895, 7941, 7985, 8046, 8107, 8169, 8232, 8280
];


pub fn from_mod(module: Module) -> Result<Module, Error> {

    let data = module.data.as_any().downcast_ref::<ModData>().unwrap();

    let mut ins_num = 0;
    for i in 0..31 {
        if data.instruments[i].size > 0 {
            ins_num = i + 1
        }
    }

    let mut instruments = Vec::<S3mInstrument>::new();
    for i in 0..ins_num {
        instruments.push(S3mInstrument{
            typ     : 1,
            memseg  : 0,
            length  : data.instruments[i].size as u32 * 2,
            loop_beg: data.instruments[i].repeat as u32 * 2,
            loop_end: (data.instruments[i].repeat as u32 + data.instruments[i].replen as u32) * 2,
            vol     : data.instruments[i].volume as i8,
            flags   : if data.instruments[i].replen > 1 { 1 } else { 0 },
            c2spd   : FINETUNE_TABLE[(data.instruments[i].finetune & 0x0f) as usize] as u32,
            name    : data.instruments[i].name.clone(),
        });
    }

    let ch = match module.format_id {
        "m.k." => 4,
        "6CHN" => 6,
        "8CHN" => 8,
        _      => return Err(Error::Format(format!("can't import {} module", module.format_id))),
    };

    let pat_num = data.patterns.num();
    let mut patterns = Vec::<S3mPattern>::new();

    for i in 0..pat_num {
        patterns.push(encode_pattern(&data.patterns, i as usize, ch))

    }

    let new_data = S3mData{
        song_name  : data.song_name.clone(),
        ord_num    : data.song_length as u16,
        ins_num    : ins_num as u16,
        pat_num    : pat_num as u16,
        flags      : 0,
        cwt_v      : 0x1320,  // Scream Tracker 3.20
        ffi        : 1,       // signed samples
        g_v        : 64,
        i_s        : 6,
        i_t        : 125,
        m_v        : 0xb0,
        d_p        : 0xd2,    // not 0xfc
        ch_settings: [ 0,  9, 10,  3,  4, 14, 15,  7, !0, !0, !0, !0, !0, !0, !0, !0,
                      !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0, !0 ],
        orders     : data.orders.to_vec(),
        instrum_pp : vec![0xd2; ins_num],   // != 0
        pattern_pp : vec![0xd2; pat_num],   // != 0
        ch_pan     : [0; 32],
        instruments,
        patterns,
        samples    : data.samples.clone(),

        channels   : ch,
    };

    Ok(Module{
        format_id  : "m.k.",
        description: "Imported M.K. module".to_owned(),
        creator    : "Scream Tracker 3".to_owned(),
        channels   : ch,
        player     : "st3",
        data       : Box::new(new_data),
    })
}

fn encode_pattern(patterns: &ModPatterns, num: usize, ch: usize) -> S3mPattern {
    let mut size = 2;
    let mut data = Vec::<u8>::new();

    data.push(0);   // make room for pattern size
    data.push(0);

    for r in 0..64 {
        for c in 0..ch {
            let e = patterns.event(num, r, c);
            let mut b = 0_u8;
            if e.note != 0 {
                b |= 0x20;  // note and instrument follow
            }
            if e.cmd&0x0f == 0x0c {
                b |= 0x40;  // volume follows
            }
            if (e.cmd&0x0f != 0 || e.cmdlo != 0) && e.cmd&0x0f != 0x0c {
                b |= 0x80;  // command and info follow
            }
            if b != 0 {
                b |= c as u8;     // channel
                &data.push(b); size += 1;
            }
            if b & 0x20 != 0 {
                let mut note = mk::period_to_note(e.note & 0xfff);
                note = if note == 0 {  // hi=oct, lo=note, 255=empty note
                    255
                } else {
                    ((note/12)-1)<<4 | note%12
                };

                let ins = ((e.note&0xf000) >> 8) as u8 | (e.cmd&0xf0) >> 4;
                &data.push(note); size += 1;
                &data.push(ins); size += 1;
            }
            if b & 0x40 != 0 {
                &data.push(e.cmdlo); size += 1;
            }
            if b & 0x80 != 0 {
                let (cmd, info) = convert_cmd(e.cmd&0x0f, e.cmdlo);
                &data.push(cmd); size += 1;
                &data.push(info); size +=1;
            }
        }
        data.push(0); size += 1;
    }

    data[0] = (size & 0xff) as u8;
    data[1] = (size >> 8) as u8;

    S3mPattern{
        size,
        data,
    }
}

fn convert_cmd(cmd: u8, info: u8) -> (u8, u8) {
    let new_cmd: u8;
    let mut new_info = info;

    let x = match cmd {
        0  => {     // Normal play or Arpeggio
            if info != 0{
                'J'
            } else {
                '@'
            }
        },
        1  => {     // Slide Up
            'F'
        },
        2  => {     // Slide Down
            'E'
        },
        3  => {     // Tone Portamento
            'G'
        },
        4  => {     // Vibrato
            'H'
        },
        5  => {     // Tone Portamento + Volume Slide
            'L'
        },
        6  => {     // Vibrato + Volume Slide
            'K'
        },
        7  => {     // Tremolo
            'R'
        },
        9  => {     // Set SampleOffset
            'O'
        },
        10 => {     // VolumeSlide
            'D'
        },
        11 => {     // Position Jump
            'B'
        },
        12 => {     // Set Volume
            new_info = 0;   // already set as volume
            '@'
        },
        13 => {     // Pattern Break
            new_info = (info >> 4) * 10 + (info & 0x0f);
            'C'
        },
        14 => {     // E-Commands
            match info >> 4 {
                0  => {     // Set Filter
                    'S'
                },
                1  => {     // E1- FineSlide Up
                    new_info = 0xf0 | info&0x0f;
                    'F'
                },
                2  => {     // E2- FineSlide Down
                    new_info = 0xf0 | info&0x0f;
                    'E'
                },
                3  => {     // E3- Glissando Control
                    new_info = 0x10 | info&0x0f;
                    'S'
                },
                4  => {     // E4- Set Vibrato Waveform
                    new_info = 0x30 | info&0x0f;
                    'S'
                },
                5  => {     // E5- Set Loop
                    new_info = 0xb0;
                    'S'
                },
                6  => {     // E6- Jump to Loop
                    new_info = 0xb0 | info&0x0f;
                    'S'
                },
                7  => {     // E7- Set Tremolo Waveform
                    new_info = 0x40 | info&0x0f;
                    'S'
                },
                9  => {     // E9- Retrig Note
                    new_info = info & 0x0f;
                    'O'
                }
                10 => {     // EA- Fine VolumeSlide Up
                    new_info = (info&0x0f)<<4 | 0x0f;
                    'D'
                },
                11 => {     // EB- Fine VolumeSlide Down
                    new_info = 0xf0 | info&0x0f;
                    'D'
                },
                12 => {     // EC- NoteCut
                    'S'
                },
                13 => {     // ED- NoteDelay
                    'S'
                },
                14 => {     // EE- PatternDelay
                    'S'
                },
                15 => {     // EF- Invert Loop
                    'S'
                },
                _  => {
                    new_info = 0;
                    '@'
                }

            }
        },
        15 => {     // Set Speed
            if info < 0x20 {
                'A'
            } else {
                'T'
            }
        }
        _  => {
            '@'
        },
    };

    new_cmd = x as u8 - '@' as u8;

    (new_cmd, new_info)
}
