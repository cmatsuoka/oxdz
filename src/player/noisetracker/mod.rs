mod player;

use module::Module;
use player::{PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct Nt11;

impl PlayerListEntry for Pt21a {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "nt11",
          name       : "Noisetracker v1.1 replayer",
          description: "A mod player based on the on the original Noisetracker V1.1 replayer",
          author     : "Claudio Matsuoka",
          accepts    : &[ "mod" ],
       }
   }

   fn player(&self, module: &Module) -> Box<FormatPlayer> {
       Box::new(self::player::ModPlayer::new(module))
   }
}


