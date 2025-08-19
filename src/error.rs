use crate::error::EngineError::AutoError;
use crate::parser::parser::ParserError;
use slint::PlatformError;
use std::io::Error;
use std::num::ParseIntError;
use serde::Deserialize;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Deserialize)]
pub enum EngineError {
    #[warn(dead_code)]
    FileError,
    ParseError,
    UiError,
    AutoError,
    ConfigError,
}

impl From<ParserError> for EngineError {
    fn from(_: ParserError) -> Self {
        EngineError::ParseError
    }
}

impl From<Error> for EngineError {
    fn from(_: Error) -> Self {
        EngineError::FileError
    }
}

impl From<PlatformError> for EngineError {
    fn from(_: PlatformError) -> Self {
        EngineError::UiError
    }
}

impl From<ParseIntError> for EngineError {
    fn from(_: ParseIntError) -> Self {
        EngineError::ParseError
    }
}

impl<T> From<SendError<T>> for EngineError {
    fn from(_: SendError<T>) -> Self {
        AutoError
    }
}

impl From<toml::de::Error> for EngineError {
    fn from(_: toml::de::Error) -> Self {
        EngineError::ConfigError
    }
}
