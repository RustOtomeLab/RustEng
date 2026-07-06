use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum EngineError {
    #[error("script error: {0}")]
    Script(#[from] ScriptError),

    #[error("executors error: {0}")]
    Executor(#[from] ExecutorError),

    #[error("save error: {0}")]
    Save(#[from] SaveError),

    #[error("media error: {0}")]
    Media(#[from] MediaError),

    #[error("ui error: {0}")]
    Ui(#[from] slint::PlatformError),
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for EngineError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        EngineError::Executor(ExecutorError::ChannelClosed)
    }
}

impl<T> From<tokio::sync::mpsc::error::TrySendError<T>> for EngineError {
    fn from(_: tokio::sync::mpsc::error::TrySendError<T>) -> Self {
        EngineError::Executor(ExecutorError::ChannelFulled)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ScriptError {
    #[error("invalid command at line {line}: {content}")]
    InvalidCommand { line: usize, content: String },

    #[error("malformed dialogue at line {line}: {content}")]
    MalformedDialogue { line: usize, content: String },

    #[error("unknown line at line {line}: {content}")]
    UnknownLine { line: usize, content: String },

    #[error("unsupported script version: need {need}, got `{indeed}`")]
    UnsupportedVersion { need: usize, indeed: String },

    #[error("invalid choice block: {0}")]
    Choice(String),

    #[error("command `{cmd}` at line {line} requires more arguments: {content}")]
    ArgsTooShort {
        cmd: String,
        line: usize,
        content: String,
    },

    #[error("failed to parse integer in script: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("failed to read script file `{path}`: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub(crate) enum ExecutorError {
    #[error("internal channel closed")]
    ChannelClosed,

    #[error("internal channel fulled")]
    ChannelFulled,

    #[error("CG metadata not found for id {0}")]
    CgMetadataMissing(u64),

    #[allow(dead_code)]
    #[error("invalid executors state: {0}")]
    InvalidState(&'static str),
}

#[derive(Debug, Error)]
pub(crate) enum SaveError {
    #[allow(dead_code)]
    #[error("failed to read save file `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write save file `{path}`: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize save data: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("failed to deserialize save data `{path}`: {source}")]
    Deserialize {
        path: String,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Debug, Error)]
pub(crate) enum MediaError {
    #[error("failed to open media file `{path}`: {source}")]
    OpenFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to decode audio file `{path}`: {source}")]
    DecodeAudio {
        path: String,
        #[source]
        source: rodio::decoder::DecoderError,
    },

    #[error("failed to create audio output stream: {0}")]
    OutputStream(#[from] rodio::StreamError),

    #[error("failed to create audio sink: {0}")]
    Sink(#[from] rodio::PlayError),

    #[allow(dead_code)]
    #[error("failed to decode video `{path}`: {reason}")]
    DecodeVideo { path: String, reason: String },
}
