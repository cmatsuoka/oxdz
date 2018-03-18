use player::State;

pub trait SaveRestore {
    unsafe fn save(&self) -> Vec<u8>;
    unsafe fn restore(&mut self, &Vec<u8>);
}

#[derive(Default, Clone)]
pub struct ScanData {
    pub ord  : usize,
    pub row  : usize,
    pub frame: usize,
    pub num  : usize,
}

#[derive(Clone)]
pub struct OrdData {
    pub state: State,
    pub time : u32,
    pub used : bool,
}

impl OrdData {
    pub fn new() -> Self {
        OrdData{
            state: vec![0; 0],
            time : 0,
            used : false,
        }
    }
}
