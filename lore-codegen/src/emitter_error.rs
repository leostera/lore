use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Diagnostic, Error, Debug, PartialEq, Eq)]
#[error("Error emitting file {filename:?}")]
#[diagnostic(code(lore::codegen::emitter), url(docsrs))]
pub struct EmitterError {
    filename: PathBuf,
}
