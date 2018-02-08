mod player;

use module::Module;
use player::{Options, PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct Ust27;

impl PlayerListEntry for Ust27 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "ust",
          name       : "Ultimate Soundtracker V27 replayer",
          description: r#"Port of the Ultimate Soundtracker replayer version 27 "All bugs removed" (29.03.1988)"#,
          author     : "Claudio Matsuoka",
          accepts    : &[ "m15" ],
       }
   }

   fn player(&self, module: &Module, options: Options) -> Box<FormatPlayer> {
       Box::new(self::player::ModPlayer::new(module, options))
   }
}


