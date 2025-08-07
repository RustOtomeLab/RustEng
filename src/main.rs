mod parser;
mod executor;
mod error;
mod script;
mod ui;
mod audio;

use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use crate::audio::player::BgmPlayer;
use crate::error::EngineError;
use crate::parser::script_parser::parse_script;
use crate::ui::ui::ui;

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
        Args { path: "./script/ky01.reg".to_string() }
    }
}

#[tokio::main]
async fn main() -> Result<(), EngineError> {
    let script_file = Args::default();
    let script = fs::read_to_string(&script_file.path)?;
    let script = Rc::new(RefCell::new(parse_script(&script)?));
    let bgm_player = Rc::new(RefCell::new(BgmPlayer::new()));
    //println!("{:#?}", script);
    ui(script, bgm_player).await?;
    Ok(())
}
