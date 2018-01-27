use util::NOTES;

pub const HAS_NOTE: u8 = 0x01;
pub const HAS_INS : u8 = 0x02;
pub const HAS_VOL : u8 = 0x04;
pub const HAS_CMD : u8 = 0x08;

pub fn format(b: &[u8]) -> String {
    let note = if b[0] & HAS_NOTE != 0 {
        format!("{}{}", NOTES[b[1] as usize % 12], b[1] / 12)
    } else {
        "---".to_owned()
    };

    let ins = if b[0] & HAS_INS != 0 {
        format!("{:02x}", b[2])
    } else {
        "--".to_owned()
    };

    let vol = if b[0] & HAS_VOL != 0 {
        format!("{:02x}", b[3])
    } else {
        "--".to_owned()
    };

    format!("{} {} {} {:02X}{:02X}", note, ins, vol, b[4], b[5])
}

