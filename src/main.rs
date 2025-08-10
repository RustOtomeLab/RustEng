mod audio;
mod error;
mod executor;
mod parser;
mod script;
mod ui;

use crate::audio::player::Player;
use crate::error::EngineError;
use crate::ui::ui::ui;
use std::cell::RefCell;
use std::rc::Rc;
use crate::script::Script;

#[tokio::main]
async fn main() -> Result<(), EngineError> {
    let script = Rc::new(RefCell::new(Script::from_name("ky01".to_string())?));
    let bgm_player = Rc::new(RefCell::new(Player::new()));
    let voice_player = Rc::new(RefCell::new(Player::new()));
    //println!("{:#?}", script);
    ui(script, bgm_player, voice_player).await?;
    Ok(())
}
