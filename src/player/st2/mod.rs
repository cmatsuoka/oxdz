mod st2play;

use module::Module;
use player::{PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct St2;

impl PlayerListEntry for St2 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "st2",
          name       : "st2play replayer",
          description: "An accurate port of the Scream Tracker 2.xx replayer",
          author     : "",
          accepts    : &[ "stm" ],
       }
   }

   fn player(&self, module: &Module) -> Box<FormatPlayer> {
       Box::new(self::st2play::St2Play::new(module))
   }
}


