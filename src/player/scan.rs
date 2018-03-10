pub trait SaveRestore {
    unsafe fn save(&self) -> Vec<u8>;
    unsafe fn restore(&mut self, Vec<u8>);
}

#[derive(Default, Clone, Copy)]
pub struct ScanData {
    pub time: usize,
    pub ord : usize,
    pub row : usize,
    pub num : usize,
}

