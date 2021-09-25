use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    name: PathBuf,
    contents: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Error writting file {filename:?}")]
#[diagnostic(code(lore::codegen::source_set), url(docsrs))]
pub struct SourceError {
    filename: PathBuf,

    #[source]
    error: std::io::Error,
}

impl Source {
    pub fn new(name: PathBuf, contents: String) -> Source {
        Source { name, contents }
    }

    pub fn write(&self, prefix: &PathBuf) -> Result<(), SourceError> {
        let path = prefix.join(self.name.clone());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| SourceError {
                filename: path.clone(),
                error,
            })?;
        }
        std::fs::write(&path, &self.contents).map_err(|error| SourceError {
            filename: path,
            error,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceSet {
    sources: Vec<Source>,
}

impl SourceSet {
    pub fn empty() -> SourceSet {
        SourceSet::default()
    }

    pub fn from_sources(sources: Vec<Source>) -> SourceSet {
        SourceSet { sources }
    }

    pub fn sources(&self) -> &Vec<Source> {
        &self.sources
    }
}
