use format::mk::ModData;

#[derive(PartialEq)]
pub enum TrackerID {
    Unknown,
    Protracker,
    Noisetracker,
    Soundtracker,
    Screamtracker3,
    FastTracker,
    FlexTrax,
    OpenMPT,
    Converted,
    ConvertedST,
    UnknownOrConverted,
    ProtrackerClone,
}

static STANDARD_NOTES: [u16; 36] = [
    856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
    428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
    214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113
];

/// Try to identify the tracker used to create an M.K. module. This is a direct port of the
/// mod fingerprinting routine used in libxmp. Yes, it's messy and should be refactored
/// some day.
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

            if out_of_range {
                // Check out-of-range notes in Amiga trackers
                if tracker_id == TrackerID::Protracker || tracker_id == TrackerID::Noisetracker || tracker_id == TrackerID::Soundtracker {
                    tracker_id = TrackerID::Unknown
                }

                if tracker_id == TrackerID::Unknown && data.restart == 0x7f {
                    tracker_id = TrackerID::Screamtracker3
                }

            }
        }

        tracker_id
    }

    fn get_tracker_id(data: &ModData) -> TrackerID {
        let mut tracker_id = match data.magic.as_ref() {
            "M.K." => TrackerID::Protracker,
            "M!K!" => return TrackerID::Protracker,
            "M&K!" => return TrackerID::Noisetracker,
            "N.T." => return TrackerID::Noisetracker,
            _      => TrackerID::Unknown,
        };
    
        if Fingerprint::has_large_instruments(&data) {
            return TrackerID::OpenMPT;
        }
            
        // Test for Flextrax modules
    
        // Test for Mod's Grave WOW modules
    
        let has_replen_0 = Fingerprint::has_replen_0(&data);
        let has_st_instruments = Fingerprint::has_st_instruments(&data);
        let empty_ins_has_volume = Fingerprint::empty_ins_has_volume(&data);
    
        if data.restart as usize == data.patterns.num {
            tracker_id = TrackerID::Soundtracker;
        } else if data.restart == 0x78 {
            // Not really sure, "MOD.Data City Remix" has Protracker effects and Noisetracker restart byte
            return TrackerID::Noisetracker;
        } else if data.restart < 0x7f {
            tracker_id = if empty_ins_has_volume {
                TrackerID::Unknown
            } else {
                TrackerID::Noisetracker
            }
            // FIXME: assume restart as noisetracker restart
        } else if data.restart == 0x7f {
            if has_replen_0 {
                tracker_id = TrackerID::ProtrackerClone;
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
                if Fingerprint::empty_ins_replen_1(&data) {
                    if empty_ins_has_volume {
                        return TrackerID::OpenMPT
                    } else if Fingerprint::only_nt_cmds(&data) && Fingerprint::standard_notes(&data) {
                        return TrackerID::Noisetracker
                    }
                }
                tracker_id = if Fingerprint::standard_octaves(&data) {
                    TrackerID::Protracker
                } else {
                    TrackerID::Unknown
                }
            }
        } else {  // Has loops with size 0
            if !Fingerprint::has_ins_15_to_31(&data) {
                return  TrackerID::ConvertedST;
            }
            if has_st_instruments {
                return TrackerID::UnknownOrConverted;
            } else {
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

    fn empty_ins_replen_1(data: &ModData) -> bool {
        for ins in &data.instruments {
            if ins.size == 0 && ins.replen == 1 {
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
