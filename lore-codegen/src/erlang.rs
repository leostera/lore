use crate::emitter_error::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErlangType {
    name: String,
}

impl std::fmt::Display for ErlangType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "-type {}().", self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErlangModule {
    name: String,
    types: Vec<ErlangType>,
}

impl std::fmt::Display for ErlangModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "-module({}).\n", self.name)?;
        for t in &self.types {
            write!(f, "{}\n", t)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct ErlangEmitter {}

// impl LoreEmitter<Document> for ErlangEmitter {
impl ErlangEmitter {
    pub fn new() -> ErlangEmitter {
        ErlangEmitter::default()
    }

    pub fn translate(&self, store: &lore_store::Store) -> Result<ErlangModule, EmitterError> {
        let mut types = vec![];

        for kind in store.kinds() {
            let name = kind.name.to_string().replace(":", "_").replace("/", "__");

            types.push(ErlangType { name });
        }

        Ok(ErlangModule {
            name: "Ontology".to_string(),
            types,
        })
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
                let emitter = ErlangEmitter::new();
                let caml_value = emitter.translate(&store).unwrap();
                let snapshot = format!(
                    r#"
input:
    {}

output:

{}
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

use spotify:ontology:2022/Artist as Artist
use spotify:ontology:2022/Album as Album
use spotify:ontology:2022/Track as Track
use spotify:ontology:2022/Name as Name
use spotify:ontology:2022/hasOne as hasOne
use spotify:ontology:2022/isListedIn as isListedIn

kind Artist

kind Album

kind Track

attr Name

rel Album hasOne Name

rel Track isListedIn Album

        "#
    );
}
