mod st2play;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};
use ::*;

pub struct St2;

impl PlayerListEntry for St2 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "st2",
          name       : "st2play(ox) ST2.21 replayer",
          description: "A port of the Scream Tracker 2.21 replayer",
          author     : r#"Sergei "x0r" Kolzun, Claudio Matsuoka"#,
          accepts    : &[ "stm" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::st2play::St2Play::new(module, options))
   }

   fn import(&self, module: Module) -> Result<Module, Error> {
       Ok(module)
   }
}


