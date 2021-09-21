use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    name: PathBuf,
    contents: String,
}

impl Source {
    pub fn new(name: PathBuf, contents: String) -> Source {
        Source { name, contents }
    }

    pub fn write(&self, prefix: &PathBuf) -> Result<(), std::io::Error> {
        let path = prefix.join(self.name.clone());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, &self.contents)
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
