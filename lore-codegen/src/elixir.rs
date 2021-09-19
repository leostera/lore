use crate::emitter_error::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElixirType {
    name: String,
}

impl std::fmt::Display for ElixirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "@type {}()", self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElixirModule {
    name: String,
    types: Vec<ElixirType>,
}

impl std::fmt::Display for ElixirModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "defmodule {} do\n", self.name)?;
        for t in &self.types {
            write!(f, "{}\n", t)?;
        }
        write!(f, "end")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElixirLib {
    modules: Vec<ElixirModule>,
}

impl std::fmt::Display for ElixirLib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        for m in &self.modules {
            write!(f, "{}\n", m)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct ElixirEmitter {}

// impl LoreEmitter<Document> for ElixirEmitter {
impl ElixirEmitter {
    pub fn new() -> ElixirEmitter {
        ElixirEmitter::default()
    }

    pub fn translate(&self, store: &lore_store::Store) -> Result<ElixirLib, EmitterError> {
        let mut modules = vec![];

        for kind in store.kinds() {
            let mut name = kind.name.to_string().replace(":", "_").replace("/", ".");
            if let Some(first) = name.get_mut(0..1) {
                first.make_ascii_uppercase();
            }

            modules.push(ElixirModule {
                name,
                types: vec![ElixirType {
                    name: "t".to_string(),
                }],
            });
        }

        Ok(ElixirLib { modules })
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
                let emitter = ElixirEmitter::new();
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
