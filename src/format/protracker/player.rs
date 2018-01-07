use module::{Module, FormatPlayer};
use player::PlayerData;
use player::Virtual;
use super::ModPatterns;

const FX_TONEPORTA: u8 = 0x03;

pub struct ModPlayer {
    name : &'static str,
    state: Vec<ChannelData>,
}

impl ModPlayer {
    pub fn new(module: &Module) -> Self {
        ModPlayer {
            name : "Protracker module player",
            state: vec![ChannelData::new(); module.chn],
        }
    }

    fn play_event(&mut self, data: &mut PlayerData, chn: usize, module: &Module, pats: &ModPatterns, virt: &mut Virtual) {

        let (pos, row, frame) = (data.pos, data.row, data.frame);
        let pat = module.orders.pattern(data);
        let xc = &mut self.state[chn];

        let event = pats.event(pos, row, chn);

        println!("play event: pos:{} pat:{} row:{} frame:{} channel:{} : {}",
            pos, pat, row, frame, chn, event);

        // Check if instrument is valid
        if event.ins as usize >= module.instrument.len() {
            return;
        }

        if data.frame == 0 {
            if event.has_ins() {
                if event.fxt != FX_TONEPORTA {
                    xc.ins = event.ins - 1;
                }
            }
            if event.has_note() {
                if event.fxt != FX_TONEPORTA {
                    xc.key = event.note - 1;
                    if xc.ins > 0 {
                        if module.sample[xc.ins as usize].size > 0 {
                            virt.set_patch(chn, xc.ins as usize, xc.ins as usize, xc.key as usize);
                        }
                    }
                }
            }
        } else {
            
        }
    }

}

impl FormatPlayer for ModPlayer {
    fn name(&self) -> &'static str {
        self.name
    }

    fn play(&mut self, mut data: &mut PlayerData, module: &Module, mut virt: &mut Virtual) {
        let pats = module.patterns.as_any().downcast_ref::<ModPatterns>().unwrap();

        for chn in 0..module.chn {
            self.play_event(&mut data, chn, &module, &pats, &mut virt)
        }
    }
}


#[derive(Clone)]
struct ChannelData {
    key   : u8,
    ins   : u8,
    period: f64,
}

impl ChannelData {
    pub fn new() -> Self {
        ChannelData {
            key   : 0,
            ins   : 0,
            period: 0_f64,
        }
    }
}
