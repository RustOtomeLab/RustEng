mod audio;
mod error;
mod executor;
mod parser;
mod script;
mod ui;

use crate::audio::player::Player;
use crate::error::EngineError;
use crate::parser::script_parser::parse_script;
use crate::ui::ui::ui;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

struct Args {
    path: String,
}

impl Args {
    fn new(args: &str) -> Args {
        Args {
            path: format!("./script/{}.reg", args),
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Args {
            path: "./script/ky01.reg".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), EngineError> {
    let script_file = Args::default();
    let script = fs::read_to_string(&script_file.path)?;
    let script = Rc::new(RefCell::new(parse_script(&script)?));
    let bgm_player = Rc::new(RefCell::new(Player::new()));
    let voice_player = Rc::new(RefCell::new(Player::new()));
    //println!("{:#?}", script);
    ui(script, bgm_player, voice_player).await?;
    Ok(())
}
