use module::{Module, PlayFrame};
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
}

impl PlayFrame for ModPlayer {
    fn name(&self) -> &'static str {
        self.name
    }

    fn play(&self, player: &Player, module: &Module) {
        let pat = module.patterns.as_any().downcast_ref::<ModPatterns>().unwrap();
    }
}
