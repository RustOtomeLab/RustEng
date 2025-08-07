use crate::parser::script_parser::ParserError;
use slint::PlatformError;
use std::io::Error;

#[derive(Debug)]
pub enum EngineError {
    FileError(Error),
    ParseError(ParserError),
    UiError(PlatformError),
}

impl From<ParserError> for EngineError {
    fn from(err: ParserError) -> Self {
        EngineError::ParseError(err)
    }
}

impl From<Error> for EngineError {
    fn from(err: Error) -> Self {
        EngineError::FileError(err)
    }
}

impl From<PlatformError> for EngineError {
    fn from(err: PlatformError) -> Self {
        EngineError::UiError(err)
    }
}
