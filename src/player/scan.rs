
#[derive(Default)]
pub struct ScanRow {
    millis: u32,  // milliseconds since start of replay
    speed : u8,   // current replay speed
    bpm   : u8,   // current replay bpm
    gvol  : u8,   // current global volume
}


pub struct ScanPos {
    row: Vec<ScanRow>,
}

pub struct ScanData {
    pos: Vec<ScanPos>,
}

impl ScanData {
    pub fn new(size: usize) -> Self {
        ScanData {
            pos: Vec::new(),
        }
    }
}

