use module::{Module, FormatPlayer};
use player::Player;
use super::ModPatterns;

pub struct ModPlayer {
    name: &'static str,
}

impl ModPlayer {
    pub fn new() -> Self {
        ModPlayer {
            name: "Protracker frame player",
        }
    }

    fn play_event(&self, player: &Player, chn: usize, module: &Module, pats: &ModPatterns) {

        let (pos, row, frame) = (player.position(), player.row(), player.frame());
        let pat = module.orders.pattern(player);

        let event = pats.event(pos, row, chn);

        println!("play event: pos:{} pat:{} row:{} frame:{} channel:{} : {}",
            pos, pat, row, frame, chn, event);

        // Check if instrument is valid
        if event.ins as usize >= module.instrument.len() {
            return;
        }

        self.process_volume(chn);
        self.process_period(chn);
        self.update_volume(chn);
        self.update_period(chn);
    }

    fn process_volume(&self, chn: usize) {
    }

    fn process_period(&self, chn: usize) {
    }

    fn update_volume(&self, chn: usize) {
    }

    fn update_period(&self, chn: usize) {
    }
}

impl FormatPlayer for ModPlayer {
    fn name(&self) -> &'static str {
        self.name
    }

    fn play(&self, player: &Player, module: &Module) {
        let pats = module.patterns.as_any().downcast_ref::<ModPatterns>().unwrap();

        for chn in 0..module.chn {
            self.play_event(&player, chn, &module, &pats)
        }
    }
}
