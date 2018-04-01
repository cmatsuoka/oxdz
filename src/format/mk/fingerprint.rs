use format::mk::ModData;

#[derive(PartialEq, Clone, Debug)]
pub enum TrackerID {
    Unknown,
    Protracker,
    Noisetracker,
    Soundtracker,
    Screamtracker3,
    FastTracker,
    FastTracker2,
    Octalyser,
    TakeTracker,
    DigitalTracker,
    ModsGrave,
    FlexTrax,
    OpenMPT,
    Converted,
    ConvertedST,
    UnknownOrConverted,
    ProtrackerClone,
}

struct Magic {
    magic: &'static str,
    flag : bool,
    id   : TrackerID,
    ch   : u8,
}

lazy_static! {
    static ref MAGIC: Box<[Magic; 13]> = Box::new([
        Magic{magic:"M.K.", flag:false, id:TrackerID::Protracker,     ch:4},
        Magic{magic:"M!K!", flag:true,  id:TrackerID::Protracker,     ch:4},
        Magic{magic:"M&K!", flag:true,  id:TrackerID::Noisetracker,   ch:4},
        Magic{magic:"N.T.", flag:true,  id:TrackerID::Noisetracker,   ch:4},
        Magic{magic:"6CHN", flag:false, id:TrackerID::FastTracker,    ch:6},
        Magic{magic:"8CHN", flag:false, id:TrackerID::FastTracker,    ch:8},
        Magic{magic:"CD61", flag:true,  id:TrackerID::Octalyser,      ch:6},  // Atari STe/Falcon
        Magic{magic:"CD81", flag:true,  id:TrackerID::Octalyser,      ch:8},  // Atari STe/Falcon
        Magic{magic:"TDZ4", flag:true,  id:TrackerID::TakeTracker,    ch:4},  // see XModule SaveTracker.c
        Magic{magic:"FA04", flag:true,  id:TrackerID::DigitalTracker, ch:4},  // Atari Falcon
        Magic{magic:"FA06", flag:true,  id:TrackerID::DigitalTracker, ch:6},  // Atari Falcon
        Magic{magic:"FA08", flag:true,  id:TrackerID::DigitalTracker, ch:8},  // Atari Falcon
        Magic{magic:"NSMS", flag:true,  id:TrackerID::Unknown,        ch:4},  // in Kingdom.mod
    ]);

    static ref STANDARD_NOTES: Box<[u16; 36]> = Box::new([
        856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
        428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
        214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113
    ]);
}

/// Try to identify the tracker used to create a module. This is a direct port of the
/// mod fingerprinting routine used in libxmp.
pub struct Fingerprint;

impl Fingerprint {
    pub fn id(data: &ModData) -> TrackerID {
        let mut tracker_id = Self::get_tracker_id(&data);
        let mut out_of_range = false;

        for p in 0..*&data.patterns.num {
            for r in 0..64 {
                for c in 0..4 {
                    let e = data.patterns.event(p, r, c);
                    let note = e.note & 0xfff;
                    let cmd = e.cmd & 0x0f;

                    if note != 0 && (note < 109 || note > 907) {
                        out_of_range = true
                    }

                    // Filter Noisetracker events
                    if tracker_id == TrackerID::Noisetracker {
                        if (cmd > 0x06 && cmd < 0x0a) || (cmd == 0x0e && e.cmdlo > 1) {
                            tracker_id = TrackerID::Unknown
                        }
                    }
                }
            }

            if tracker_id == TrackerID::Noisetracker {
                if !Fingerprint::only_nt_cmds(&data) || !Fingerprint::standard_notes(&data) {
                    tracker_id = TrackerID::Unknown
                }
            } else if tracker_id == TrackerID::Soundtracker {
                if !Fingerprint::standard_notes(&data) {
                    tracker_id = TrackerID::Unknown
                }
            } else if tracker_id == TrackerID::Protracker {
                if !Fingerprint::standard_octaves(&data) {
                    tracker_id = TrackerID::Unknown
                }
            }

            if out_of_range {
                if tracker_id == TrackerID::Unknown && data.restart == 0x7f {
                    tracker_id = TrackerID::Screamtracker3
                }
            }
        }

        tracker_id
    }

    fn get_tracker_id(data: &ModData) -> TrackerID {

        let mut tracker_id = TrackerID::Unknown;
        let mut detected = false;
        let mut chn = 0;

        for m in MAGIC.iter() {
            if data.magic == m.magic {
                tracker_id = m.id.clone();
                chn        = m.ch;
                detected   = m.flag;
                break;
            }
        }

        if detected {
            return tracker_id;
        }

        if chn == 0 {
            let magic: Vec<char> = data.magic.chars().collect();
            if magic[0].is_digit(10) && magic[1].is_digit(10) && &data.magic[2..] == "CH" {
                chn = (magic[0] as u8 - '0' as u8) * 10 + magic[1] as u8 - '0' as u8;
            } else if magic[0].is_digit(10) && &data.magic[1..] == "CHN" {
                chn = magic[0] as u8 - '0' as u8;
            } else {
                return TrackerID::Unknown;
            }

            return if chn&1 != 0 { TrackerID::TakeTracker } else { TrackerID::FastTracker2 };
        }

        if Fingerprint::has_large_instruments(&data) {
            return TrackerID::OpenMPT;
        }

        let has_replen_0 = Fingerprint::has_replen_0(&data);
        let has_st_instruments = Fingerprint::has_st_instruments(&data);
        let empty_ins_has_volume = Fingerprint::empty_ins_has_volume(&data);

        if data.restart as usize == data.patterns.num {
            tracker_id = if chn == 4 {
                TrackerID::Soundtracker
            } else {
                TrackerID::Unknown
            }
        } else if data.restart == 0x78 {
            tracker_id = if chn == 4 {
                // Not really sure, "MOD.Data City Remix" has Protracker effects and Noisetracker restart byte
                TrackerID::Noisetracker
            } else {
                TrackerID::Unknown
            };
            return tracker_id
        } else if data.restart < 0x7f {
            tracker_id = if chn == 4 && !empty_ins_has_volume {
                TrackerID::Noisetracker
            } else {
                TrackerID::Unknown
            }
        } else if data.restart == 0x7f {
            if chn == 4 {
                if has_replen_0 {
                    tracker_id = TrackerID::ProtrackerClone;
                }
            } else {
                tracker_id = TrackerID::Screamtracker3;
            }
            return tracker_id;
        } else if data.restart > 0x7f {
            return TrackerID::Unknown;
        }

        if !has_replen_0 {  // All loops are size 2 or greater
            if Fingerprint::size_1_and_volume_0(&data) {
                return TrackerID::Converted;
            }

            if !has_st_instruments {
                for ins in &data.instruments {
                    if ins.size != 0 || ins.replen != 1 {
                        continue
                    }

                    tracker_id = match chn {
                        4 => {
                            if empty_ins_has_volume {
                                TrackerID::OpenMPT
                            } else {
                                TrackerID::Noisetracker  // or Octalyser
                            }
                         },
                         6 => TrackerID::Octalyser,
                         8 => TrackerID::Octalyser,
                         _ => TrackerID::Unknown,
                    };
                    return tracker_id
                }

                tracker_id = match chn {
                    4 => TrackerID::Protracker,
                    6 => TrackerID::FastTracker,  // FastTracker 1.01?
                    8 => TrackerID::FastTracker,  // FastTracker 1.01?
                    _ => TrackerID::Unknown,
                }
            }
        } else {  // Has loops with size 0
            if !Fingerprint::has_ins_15_to_31(&data) {
                return  TrackerID::ConvertedST;
            }
            if has_st_instruments {
                return TrackerID::UnknownOrConverted;
            } else if chn == 6 || chn == 8 {
                return TrackerID::FastTracker;
            }
        }

        tracker_id 
    }

    fn standard_octaves(data: &ModData) -> bool {
        for p in 0..*&data.patterns.num {
            for r in 0..64 {
                for c in 0..4 {
                    let e = data.patterns.event(p, r, c);
                    let note = e.note & 0xfff;
                    if note != 0 && (note < 109 || note > 907) {
                        return false 
                    }
                }
            }
        }
        true
    }

    fn standard_notes(data: &ModData) -> bool {
        for p in 0..*&data.patterns.num {
            for r in 0..64 {
                for c in 0..4 {
                    let e = data.patterns.event(p, r, c);
                    let note = e.note & 0xfff;
                    if note != 0 && !STANDARD_NOTES.contains(&note) {
                        return false 
                    }
                }
            }
        }
        true
    }

    fn only_nt_cmds(data: &ModData) -> bool {
        for p in 0..*&data.patterns.num {
            for r in 0..64 {
                for c in 0..4 {
                    let e = data.patterns.event(p, r, c);
                    let cmd = e.cmd & 0x0f;
                    if (cmd > 0x06 && cmd < 0x0a) || (cmd == 0x0e && e.cmdlo > 1) {
                        return false 
                    }
                }
            }
        }
        true
    }

    fn has_large_instruments(data: &ModData) -> bool {
        for ins in &data.instruments {
            if ins.size > 0x8000 {
                return true
            }
        }
        false
    }

    // Check if has instruments with repeat length 0 
    fn has_replen_0(data: &ModData) -> bool {
        for ins in &data.instruments {
            if ins.replen == 0 {
                return true
            }
        }
        false
    }

    // Check if has instruments with size 0 and volume > 0
    fn empty_ins_has_volume(data: &ModData) -> bool {
        for ins in &data.instruments {
            if ins.size == 0 && ins.volume > 0 {
                return true
            }
        }
        false
    }

    fn size_1_and_volume_0(data: &ModData) -> bool {
        for ins in &data.instruments {
            if ins.size == 1 && ins.volume == 0 {
                return true;
            }
        }
        false
    }

    fn has_st_instruments(data: &ModData) -> bool {
        for ins in &data.instruments {
            if ins.name.len() < 6 {
                return false;
            }

            let name: Vec<char> = ins.name.chars().collect();

            if name[0] != 's' && name[0] != 'S' {
                return false;
            }
            if name[1] != 't' && name[1] != 'T' {
                return false;
            }
            if name[2] != '-' || name[5] != ':' {
                return false;
            }
            if !name[3].is_digit(10) || !name[4].is_digit(10) {
                return false;
            }
        }
        true
    }

    fn has_ins_15_to_31(data: &ModData) -> bool {
        for ins in data.instruments.iter().skip(15) {
            if ins.name != "" || ins.size > 0 {
                return true
            }
        }
        false
    }
}
