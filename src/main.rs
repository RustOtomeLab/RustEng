mod config;
mod error;
mod executors;
mod media;
mod parser;
mod run;
mod script;
mod ui;

use crate::error::EngineError;
use crate::run::build;

#[tokio::main]
async fn main() -> Result<(), EngineError> {
    build().await
}
