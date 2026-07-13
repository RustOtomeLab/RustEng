mod config;
mod error;
mod executors;
mod media;
mod parser;
mod script;
mod ui;

use crate::error::EngineError;
use crate::ui::initialize::ui;

#[tokio::main]
async fn main() -> Result<(), EngineError> {
    ui().await
}
