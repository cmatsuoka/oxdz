mod it_music;

use module::Module;
use player::{PlayerListEntry, PlayerInfo, FormatPlayer};

pub struct It217;

impl PlayerListEntry for It217 {
   fn info(&self) -> PlayerInfo {
       PlayerInfo {
          id         : "it217",
          name       : "Impulse Tracker replayer",
          description: "IT module replayer based on IT 2.17",
          author     : "Claudio Matsuoka",
          accepts    : &[ "it", "s3m" ],
       }
   }

   fn player(&self, module: &Module) -> Box<FormatPlayer> {
       Box::new(self::it_music::ItPlayer::new(module))
   }
}


