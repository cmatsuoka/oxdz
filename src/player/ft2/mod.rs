mod ft2play;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct Ft2;

impl PlayerListEntry for Ft2 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "ft2",
          name       : "ft2play 0.86 replayer",
          description: "A port of the Fast Tracker 2.09a replayer",
          author     : r#"Olav "8bitbubsy" Sørensen, Claudio Matsuoka"#,
          accepts    : &[ "xm", "m.k.", "xxch" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::ft2play::Ft2Play::new(module, options))
   }
}
