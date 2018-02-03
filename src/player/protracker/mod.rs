mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct Pt21a;

impl PlayerListEntry for Pt21a {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "pt21",
          name       : "Protracker 2.1A replayer",
          description: "A mod player based on the on the original Protracker 2.1A replayer",
          author     : "Claudio Matsuoka",
          accepts    : &[ "mod" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::player::ModPlayer::new(module, options))
   }
}


