use crate::emitter_error::*;
use crate::source_set::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CamlName {
    parts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CamlModule {
    name: CamlName,
    structure: Vec<CamlValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CamlValue {
    Module(CamlModule),
    Type { name: String },
}

impl CamlName {
    pub fn from_kind_name(name: &lore_ast::Name) -> CamlName {
        let mut parts = vec![];
        for part in name.to_string().to_lowercase().replace("/", ":").split(":") {
            parts.push(part.to_string())
        }

        if let Some(first) = parts[0].get_mut(0..1) {
            first.make_ascii_uppercase();
        };

        CamlName { parts }
    }

    pub fn to_filename(&self) -> String {
        self.parts.join("_")
    }
}

impl Into<Source> for CamlModule {
    fn into(self) -> Source {
        let filename = format!("{}.mli", self.name.to_filename());
        let contents = format!("{}", self);
        Source::new(filename.into(), contents)
    }
}

impl std::fmt::Display for CamlName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.parts[0])?;
        for part in self.parts[1..].iter() {
            write!(f, "_{}", part)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for CamlModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        for item in &self.structure {
            write!(f, "{}\n", item)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for CamlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            CamlValue::Module(m) => write!(f, "{}", m),

            CamlValue::Type { name } => write!(f, "type {}", name),
        }
    }
}

#[derive(Default)]
pub struct OCamlEmitter {}

// impl LoreEmitter<Document> for OCamlEmitter {
impl OCamlEmitter {
    pub fn new() -> OCamlEmitter {
        OCamlEmitter::default()
    }

    pub fn translate(&self, store: &lore_store::Store) -> Result<SourceSet, EmitterError> {
        let mut sources = vec![];

        for kind in store.kinds() {
            let module = CamlModule {
                name: CamlName::from_kind_name(&kind.name),
                structure: vec![CamlValue::Type {
                    name: "t".to_string(),
                }],
            };
            sources.push(module.into())
        }

        Ok(SourceSet::from_sources(sources))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::*;

    macro_rules! test {
        ($name:ident, $src:expr) => {
            #[test]
            fn $name() {
                let mut store = lore_store::Store::new();
                let store = store.add_from_string($src).unwrap();
                let emitter = OCamlEmitter::new();
                let caml_value = emitter.translate(&store).unwrap();
                let snapshot = format!(
                    r#"
input:
    {}

output:

{:#?}
"#,
                    $src, caml_value
                );
                assert_snapshot!(snapshot)
            }
        };
    }

    test!(
        kind_to_type,
        r#"

use spotify:ontology:v2022/Artist as Artist
use spotify:ontology:v2022/Album as Album
use spotify:ontology:v2022/Track as Track
use spotify:ontology:v2022/Name as Name
use spotify:ontology:v2022/hasOne as hasOne
use spotify:ontology:v2022/isListedIn as isListedIn

kind Artist

kind Album

kind Track

attr Name

rel Album hasOne Name

rel Track isListedIn Album

        "#
    );
}
