use crate::parser::parser::ParserError;
use slint::PlatformError;
use std::io::Error;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum EngineError {
    #[warn(dead_code)]
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

impl From<ParseIntError> for EngineError {
    fn from(_: ParseIntError) -> Self {
        EngineError::ParseError(ParserError::ChooseError(String::from(
            "Invalid choice number",
        )))
    }
}
