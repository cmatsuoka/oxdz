mod st3play;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct St3;

impl PlayerListEntry for St3 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "st3",
          name       : "st3play 0.78",
          description: "A port of the Scream Tracker 3.21 replayer",
          author     : r#"Olav "8bitbubsy" Sørensen, Claudio Matsuoka"#,
          accepts    : &[ "s3m", "m.k.", "xxch" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::st3play::St3Play::new(module, options))
   }
}
