mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};
use ::*;

pub struct Hmn;

impl PlayerListEntry for Hmn {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "nt11",
          name       : "His Master's Noise replayer",
          description: "Based on Musicdisktrackerreplay by Mahoney December 1990",
          author     : "Claudio Matsuoka",
          accepts    : &[ "fest" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::player::HmnPlayer::new(module, options))
   }

   fn import(&self, module: Module) -> Result<Module, Error> {
       Ok(module)
   }
}


