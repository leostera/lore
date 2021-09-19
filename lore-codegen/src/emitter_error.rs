use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum EmitterError {
    #[error("Expected a URI.")]
    ExpectedURI,

    #[error("Runtime error")]
    Runtime(String),
}
