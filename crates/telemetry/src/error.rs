//! Error types for telemetry operations

use std::io;
use std::path::PathBuf;
use std::num::{ParseIntError, ParseFloatError};
use std::string::FromUtf8Error;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, TelemetryError>;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("System call error: {0}")]
    Syscall(#[from] nix::errno::Errno),

    #[error("Collection error: {0}")]
    Collection(String),

    #[error("Timeout")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid format in {path:?}: {reason}")]
    InvalidFormat { path: PathBuf, reason: String },

    #[error("Missing required field '{field}' in {path:?}")]
    MissingField { path: PathBuf, field: String },

    #[error("Invalid integer '{value}' for field '{field}' in {path:?}: {source}")]
    InvalidInteger {
        path: PathBuf,
        field: String,
        value: String,
        #[source]
        source: ParseIntError,
    },

    #[error("Invalid float '{value}' for field '{field}' in {path:?}: {source}")]
    InvalidFloat {
        path: PathBuf,
        field: String,
        value: String,
        #[source]
        source: ParseFloatError,
    },

    #[error("Invalid UTF-8 in {path:?}: {source}")]
    InvalidUtf8 {
        path: PathBuf,
        #[source]
        source: FromUtf8Error,
    },

    #[error("File too large: {path:?} ({size} bytes, limit {limit} bytes)")]
    FileTooLarge {
        path: PathBuf,
        size: usize,
        limit: usize,
    },
}

impl TelemetryError {
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn collection(msg: impl Into<String>) -> Self {
        Self::Collection(msg.into())
    }
}
