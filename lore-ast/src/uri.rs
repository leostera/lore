#[derive(Clone, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct URI(String);

impl ToString for URI {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl URI {
    pub fn unresolved() -> URI {
        URI("lore:uri:unresolved".to_string())
    }

    pub fn from_string(uri: String) -> URI {
        URI(uri)
    }

    pub fn join(&self, uri: &str) -> URI {
        URI(format!("{}/{}", self.0, uri))
    }

    pub fn is_prefixed(&self) -> bool {
        self.0.get(0..1) == Some("@")
    }

    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.0.starts_with(prefix)
    }

    pub fn expand_prefix(&self, prefix: &str, expanded: &URI) -> URI {
        URI(self.0.replace(prefix, &expanded.to_string()))
    }
}
