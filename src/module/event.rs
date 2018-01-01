use fmt;

const NOTES: &'static [&'static str] = &[
    "C ", "C#", "D ", "D#", "E ", "F ", "F#", "G ", "G#", "A ", "A#", "B "
];

#[derive(Debug)]
pub struct Event {
    pub note: u8,
    pub ins : u8,
    pub vol : u8,
    pub fxt : u8,
    pub fxp : u8,
}

impl Event {
    pub fn new() -> Self {
        Event {
            note: 0,
            ins : 0,
            vol : 0,
            fxt : 0,
            fxp : 0,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let note = if self.note == 0 {
            "---".to_owned()
        } else {
            format!("{}{}", NOTES[self.note as usize % 12], self.note / 12)
        };

        let ins = if self.ins == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.ins)
        };

        let vol = if self.vol == 0 {
            "--".to_owned()
        } else {
            format!("{:02x}", self.vol)
        };

        write!(f, "{} {} {} {:02X}{:02X}", note, ins, vol, self.fxt, self.fxp)
    }
}

