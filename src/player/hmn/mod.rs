mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};
use ::*;

pub struct Hmn;

impl PlayerListEntry for Hmn {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "hmn",
          name       : "His Master's Noise replayer",
          description: "JAG VILL HELST HA EN GET I JULKLAPP",
          author     : "Claudio Matsuoka",
          accepts    : &[ "fest", "m.k." ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::player::HmnPlayer::new(module, options))
   }

   fn import(&self, module: Module) -> Result<Module, Error> {
       Ok(module)
   }
}


