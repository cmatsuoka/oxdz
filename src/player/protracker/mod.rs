mod player;

use module::{Module, ModuleData};
use player::{PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct Pt21a;

impl PlayerListEntry for Pt21a {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "pt21",
          name       : r#""Vinterstigen" 0.1 PT2.1A replayer"#,
          description: "A mod player based on the on the original Protracker 2.1A replayer",
          author     : "Claudio Matsuoka",
          accepts    : &[ "mod" ],
       }
   }

   fn player(&self, module: &Module) -> Box<FormatPlayer> {
       Box::new(self::player::ModPlayer::new(module))
   }
}


