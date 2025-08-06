use std::io::Error;
use crate::parser::script_parser::ParserError;

#[derive(Debug)]
pub enum EngineError {
    FileError(Error),
    ParseError(ParserError),
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