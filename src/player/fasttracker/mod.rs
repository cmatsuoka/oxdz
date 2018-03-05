mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};
use ::*;

pub struct Ft101;

impl PlayerListEntry for Ft101 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "ft",
          name       : "oxdz-FT101 replayer",
          description: "Based on the FastTracker 1.01 replayer",
          author     : "Claudio Matsuoka",
          accepts    : &[ "m.k.", "6chn", "8chn" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::player::FtPlayer::new(module, options))
   }

   fn import(&self, module: Module) -> Result<Module, Error> {
       Ok(module)
   }
}


