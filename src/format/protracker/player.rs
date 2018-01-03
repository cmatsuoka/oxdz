use player::Player;
use module::{Module, PlayFrame};

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
    }
}
