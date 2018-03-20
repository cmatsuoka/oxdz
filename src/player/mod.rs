mod scan;
mod protracker;
mod noisetracker;
mod soundtracker;
mod ust;
mod st2;
mod st3;
mod hmn;
mod fasttracker;

pub use mixer::Mixer;

use std::cmp;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use module::{Module, ModuleData};
use player::scan::{ScanData, OrdData};
use util::MemOpExt;
use ::*;


fn all() -> Vec<Box<PlayerListEntry>> {
    vec![
        Box::new(protracker::Pt21a),
        Box::new(noisetracker::Nt11),
        Box::new(fasttracker::Ft101),
        Box::new(st2::St2),
        Box::new(st3::St3),
        Box::new(soundtracker::DocSt2),
        Box::new(ust::Ust27),
        Box::new(hmn::Hmn),
    ]
}

pub fn list() -> Vec<PlayerInfo> {
    all().iter().map(|p| p.info()).collect()
}

pub fn list_by_id(player_id: &str) -> Result<Box<PlayerListEntry>, Error> {
    for p in all() {
        if player_id == p.info().id {
            return Ok(p)
        }
    }
    Err(Error::Format(format!("player {:?} not found", player_id)))
}

fn accepted(player_id: &str) -> &'static [&'static str] {
    let list_entry = match list_by_id(player_id) {
        Ok(val) => val,
        Err(_)  => return &[],
    };

    list_entry.info().accepts
}

pub fn check_accepted(player_id: &str, my_fmt: &str) -> Result<bool, Error> {
    let accepted = if player_id.is_empty() {
        &[]  // accept all
    } else {
        accepted(player_id)
    };

    if accepted.is_empty() {
        return Ok(false)
    } else {
        if !accepted.contains(&my_fmt) {
           return Err(Error::Format(format!("format {:?} not accepted by player {:?}", my_fmt, player_id)))
        }
    }

    Ok(true)
}



// For the player list

pub struct PlayerInfo {
    pub id         : &'static str,
    pub name       : &'static str,
    pub description: &'static str,
    pub author     : &'static str,
    pub accepts    : &'static [&'static str],
}

pub trait PlayerListEntry {
    fn info(&self) -> PlayerInfo;
    fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer>;
    fn import(&self, module: Module) -> Result<Module, Error>;
}


// Trait for format-specific players

pub type State = Vec<u8>;

pub trait FormatPlayer: Send + Sync {
    fn start(&mut self, &mut PlayerData, &ModuleData, &mut Mixer);
    fn play(&mut self, &mut PlayerData, &ModuleData, &mut Mixer);
    fn reset(&mut self);
    unsafe fn save_state(&self) -> State;
    unsafe fn restore_state(&mut self, &State);
}

#[derive(Default)]
pub struct PlayerData {
    pub pos  : usize,
    pub row  : usize,
    pub frame: usize,
    pub song : usize,
    pub speed: usize,
    pub tempo: f32,
    pub time : f32,

    initial_speed: usize,
    initial_tempo: usize,

    loop_count: usize,
    end_point : usize,

    scan_data  : [ScanData; MAX_SEQUENCES],
    inside_loop: bool,
}

impl PlayerData {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn reset(&mut self) {
        self.pos   = 0;
        self.row   = 0;
        self.frame = 0;
        self.song  = 0;
        self.speed = self.initial_speed;
        self.tempo = self.initial_tempo as f32;
        self.time  = 0.0;
    }

    pub fn check_end_of_module(&mut self) {
        let song = self.song;
        let data = &self.scan_data[song];
        if self.pos == data.ord && self.row == data.row && self.frame == data.frame {
            if self.end_point == 0 {
                self.loop_count += 1;
                self.end_point = data.num;
            }
            self.end_point -= 1;
        }
    }

}


pub struct Player<'a> {
    pub data      : PlayerData,
    pub total_time: u32,
    module        : &'a Module,
    format_player : Box<FormatPlayer>,
    mixer         : Mixer<'a>,
    loop_count    : usize,
    end           : bool,

    // for buffer fill
    consumed      : usize,
    in_pos        : usize,
    in_size       : usize,
    
    ord_data      : Vec<OrdData>,
    scan_cnt      : Vec<Vec<u32>>,
}

impl<'a> Player<'a> {
    pub fn find(module: &'a Module, rate: u32, player_id: &str, optstr: &str) -> Result<Self, Error> {

        let list_entry = list_by_id(player_id)?;

        debug!("check if player {:?} supports format {:?}", player_id, module.format_id);
        if !list_entry.info().accepts.contains(&module.format_id) {
            return Err(Error::Format(format!("player {:?} does not support format {:?}", list_entry.info().id, player_id)))
        }

        let format_player = list_entry.player(&module, Options::from_str(optstr));

        debug!("scan counts: len={}", module.len());
        let mut scan_cnt: Vec<Vec<u32>> = Vec::new();
        for pos in 0..module.len() {
            let pat = match module.pattern_in_position(pos) {
                Some(val) => val,
                None      => break,
            };
            scan_cnt.push(vec![0; module.rows(pat as usize)]);
        }

        let mixer = Mixer::new(module.channels, rate, &module.data.samples());
        Ok(Player {
            data      : PlayerData::new(),
            module,
            format_player,
            mixer,
            total_time: 0,
            loop_count: 0,
            end       : false,
            consumed  : 0,
            in_pos    : 0,
            in_size   : 0,
            ord_data  : vec![OrdData::new(); module.len()],
            scan_cnt,
        })
    }

    pub fn scan(&mut self) -> &Self {
        let mut prev_pos = 9999;
        let mut prev_row = 9999;
        let mut prev_loop_count = 9999;

        self.format_player.start(&mut self.data, &*self.module.data, &mut self.mixer);

        loop {
            let pos = self.data.pos;
            let row = self.data.row;
            let loop_count = self.data.loop_count;

            if prev_row != row || prev_pos != pos || prev_loop_count != loop_count {

                // FIXME
                debug!("scan: check {}/{}", pos, row);
                if self.scan_cnt[pos][row] > 0 {
                    if self.data.inside_loop {
                        debug!("inside loop");
                    } else {
                        debug!("scan: already visited");
                        break;
                    }
                }


                self.scan_cnt[pos][row] += 1;
                prev_loop_count = loop_count;
                prev_row = row;

                if prev_pos != pos && !self.ord_data[pos].used {
                    unsafe{ self.ord_data[pos].state = self.format_player.save_state(); }
                    self.ord_data[pos].time = self.data.time;
                    prev_pos = pos;
                    self.ord_data[pos].used = true;
                    debug!("scan: pos {}: time {}", pos, self.ord_data[pos].time);
                }
            }

            self.format_player.play(&mut self.data, &*self.module.data, &mut self.mixer);

        }
        self.total_time = self.data.time as u32;

        debug!("end position is {}/{}", self.data.pos, self.data.row);
        let song = self.data.song;
        self.data.scan_data[song].num = self.scan_cnt[self.data.pos][self.data.row] as usize;
        self.data.scan_data[song].row = self.data.row;
        self.data.scan_data[song].ord = self.data.pos;
        self.data.scan_data[song].frame = self.data.frame;

        self.reset();
        self.data.reset();
        self.mixer.reset();

        self
    }

    pub fn reset(&mut self) {
        unsafe {
            self.format_player.restore_state(&self.ord_data[0].state);
        }
    }

    pub fn module(&self) -> &'a Module {
        self.module
    }

    pub fn set_interpolator(&mut self, name: &str) -> Result<(), Error> {
        self.mixer.set_interpolator(name)
    }

/*
    pub fn restart(&mut self) -> &Self {
        self.data.pos = 0;
        self.data.row = 0;
        self.data.song = 0;
        self.data.frame = 0;
        self.data.speed = 6;
        self.format_player.reset();
        self
    }
*/

    pub fn start(&mut self) -> &mut Self {
        self.format_player.start(&mut self.data, &*self.module.data, &mut self.mixer);

        self.data.end_point = self.data.scan_data[0].num;

        self
    }

    pub fn play_frame(&mut self) -> &mut Self {
        self.data.check_end_of_module();
        self.format_player.play(&mut self.data, &*self.module.data, &mut self.mixer);
        self.mixer.set_tempo(self.data.tempo as f64);
        self.mixer.mix();

        self
    }

    pub fn fill_buffer(&mut self, out_buffer: &mut [i16], loops: usize) {
        let mut filled = 0;
        let size = out_buffer.len();

        // Fill buffer
        while filled < size {
            // Check if buffer full
            if self.consumed == self.in_size {
                self.play_frame();

                // Check end of module
                if self.end() || (loops > 0 && self.loop_count >= loops) {
                    // Start of frame, return end of replay
                    if filled == 0 {
                        self.consumed = 0;
                        self.in_size = 0;
                        return;
                    } else {
                        self.end = false;
                    }

                    // Clear rest of the buffer
                    out_buffer[filled..].fill(0, size - filled);
                }

                self.consumed = 0;
                self.in_pos = 0;
                self.in_size = self.buffer().len();
            }

            // Copy frame data to user buffer
            let copy_size = cmp::min(size - filled, self.in_size - self.consumed);
            out_buffer[filled..filled+copy_size].copy_from_slice(&self.buffer()[self.consumed..self.consumed+copy_size]);
            self.consumed += copy_size;
            filled += copy_size;
        }
    }

    pub fn end(&self) -> bool {
        self.end
    }

    pub fn info(&mut self, info: &mut FrameInfo) -> &mut Self {
        info.pos = self.data.pos;
        info.row = self.data.row;
        info.song = self.data.song;
        info.frame = self.data.frame;
        info.speed = self.data.speed;
        info.tempo = self.data.tempo as usize;
        info.loop_count = self.data.loop_count;
        info.time = self.data.time;
        self
    }

    pub fn set_mute(&mut self, chn: usize, val: bool) {
        self.mixer.set_mute(chn, val)
    }

    pub fn set_mute_all(&mut self, val: bool) {
        self.mixer.set_mute_all(val)
    }

/*
    pub fn position(&self) -> usize {
        self.data.pos
    }

    pub fn row(&self) -> usize {
        self.data.row
    }

    pub fn frame(&self) -> usize {
        self.data.frame
    }

    pub fn song(&self) -> usize {
        self.data.song
    }

    pub fn set_row(&mut self, row: usize) -> &Self {
        self.data.row = row;
        self.set_frame(0)
    }

    pub fn set_frame(&mut self, frame: usize) -> &Self {
        self.data.frame = frame;
        self
    }
*/

    pub fn set_position(&mut self, pos: usize) -> &Self {
        if pos < self.ord_data.len() {
            unsafe{ self.format_player.restore_state(&self.ord_data[pos].state) }
            self.data.time = self.ord_data[pos].time;
        }
        self
    }

    pub fn set_song(&mut self, song: usize) -> &Self {
        self.data.song = song;
        //self.data.pos = 0; FIXME: songs may start at pos != 0
        self
    }

    pub fn buffer(&self) -> &[i16] {
        self.mixer.buffer()
    }
}


#[derive(Default)]
pub struct FrameInfo {
    pub pos  : usize,
    pub row  : usize,
    pub frame: usize,
    pub song : usize,
    pub tempo: usize,
    pub speed: usize,
    pub loop_count: usize,
    pub time : f32,
}

impl FrameInfo {
    pub fn new() -> Self {
        Default::default()
    }
}



pub struct Options {
    opt: HashMap<String, String>,
}

impl Options {
    pub fn from_str(optstr: &str) -> Self {
        let mut options = Options{
            opt: HashMap::new(),
        };

        let olist = optstr.split(";");
        for o in olist {
            if o.contains("=") {
                let kv = o.split("=").collect::<Vec<&str>>();
                let key = kv[0].trim().to_owned();
                let val = kv[1].trim().to_owned();
                options.opt.insert(key, val);
            } else {
                let key = o.trim().to_owned();
                options.opt.insert(key, "".to_owned());
            }
        }
        options
    }

    pub fn has_option(&mut self, opt: &str) -> bool {
        match self.opt.entry(opt.to_string()) {
            Entry::Occupied(_) => true,
            Entry::Vacant(_)   => false,
        }
    }

    pub fn option_int(&self, opt: &str) -> Option<isize> {
        None
    }
}


