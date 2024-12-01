use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("Pattern too complex: {0}")]
    PatternTooComplex(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}