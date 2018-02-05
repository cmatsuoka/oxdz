mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct Pt21a;

impl PlayerListEntry for Pt21a {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "pt2",
          name       : r#""Vinterstigen" PT2.1A replayer + fixes"#,
          description: "A mod player based on the on the Protracker 2.1A replayer + 2.3D fixes",
          author     : "Claudio Matsuoka",
          accepts    : &[ "mod" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::player::ModPlayer::new(module, options))
   }
}


