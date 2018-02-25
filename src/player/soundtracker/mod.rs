mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};
use ::*;

pub struct DocSt2;

impl PlayerListEntry for DocSt2 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "dst2",
          name       : "D.O.C SoundTracker V2.0",
          description: "A port of the D.O.C. SoundTracker V2.0 playroutine by Unknown of D.O.C",
          author     : "Claudio Matsuoka",
          accepts    : &[ "st" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::player::StPlayer::new(module, options))
   }

   fn import(&self, module: Module) -> Result<Module, Error> {
       Ok(module)
   }
}


