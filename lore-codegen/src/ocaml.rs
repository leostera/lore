use crate::emitter_error::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CamlValue {
    Module {
        name: String,
        structure: Vec<CamlValue>,
    },
    Type {
        name: String,
    },
}

impl std::fmt::Display for CamlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            CamlValue::Module { name, structure } => {
                write!(f, "module {} = struct\n", name)?;
                for item in structure {
                    write!(f, "  {}\n", item)?;
                }
                write!(f, "end")
            }

            CamlValue::Type { name } => {
                write!(f, "type {}", name)
            }
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

    pub fn translate(&self, store: &lore_store::Store) -> Result<CamlValue, EmitterError> {
        let mut structure = vec![];

        for kind in store.kinds() {
            let mut name = kind.name.to_string().replace(":", "_").replace("/", "__");
            if let Some(first) = name.get_mut(0..1) {
                first.make_ascii_uppercase();
            }

            let module = CamlValue::Module {
                name,
                structure: vec![CamlValue::Type {
                    name: "t".to_string(),
                }],
            };
            structure.push(module)
        }

        Ok(CamlValue::Module {
            name: "Ontology".to_string(),
            structure,
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
                let emitter = OCamlEmitter::new();
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