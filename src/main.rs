mod parser;
mod executor;
mod error;
mod script;
mod ui;

use std::fs;
use crate::error::EngineError;
use crate::executor::script_executor::execute_script;
use crate::parser::script_parser::parse_script;

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

fn main() -> Result<(), EngineError> {
    let script_file = Args::default();
    let script = fs::read_to_string(&script_file.path)?;
    let mut script = parse_script(&script)?;
    //println!("{:#?}", script);
    execute_script(&mut script);
    Ok(())
}
