use lore_ast::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error(transparent)]
    ParseError(#[from] lore_parser::ParseError),

    #[error("Many validation errors oh no")]
    ValidationError(Vec<lore_parser::ValidationError>),

    #[error("Runtime error")]
    Runtime(String),
}

#[derive(Clone, Default)]
pub struct Store {
    pub relations_by_subject: HashMap<URI, Vec<Relation>>,

    pub attributes: HashMap<URI, Attribute>,

    pub kinds: HashMap<URI, Kind>,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Store {{\n")?;

        write!(f, "  kinds: {{\n")?;
        let mut kinds: Vec<Kind> = self.kinds.values().cloned().collect();
        kinds.sort_by(|a, b| a.cmp(b));
        for k in kinds {
            write!(f, "  {:#?}\n", k)?;
        }
        write!(f, "  }}\n")?;

        write!(f, "  attributes: {{\n")?;
        let mut attributes: Vec<Attribute> = self.attributes.values().cloned().collect();
        attributes.sort_by(|a, b| a.cmp(b));
        for a in attributes {
            write!(f, "    {:#?}\n", a)?;
        }
        write!(f, "  }}\n")?;

        write!(f, "  relations: {{\n")?;
        let mut relations: Vec<(URI, Vec<Relation>)> =
            self.relations_by_subject.clone().into_iter().collect();
        relations.sort_by(|a, b| a.cmp(b));
        for rel in relations {
            write!(f, "    {:#?}\n", rel)?;
        }
        write!(f, "  }}\n")?;

        write!(f, "}}")
    }
}

impl Store {
    pub fn new() -> Store {
        Store::default()
    }

    pub fn kinds(&self) -> Vec<&Kind> {
        self.kinds.values().collect()
    }

    pub fn attributes(&self) -> Vec<&Attribute> {
        self.attributes.values().collect()
    }

    pub fn add_from_string(&mut self, src: &str) -> Result<&mut Store, StoreError> {
        let mut parser = lore_parser::Parser::for_string("tmp", src)?;
        let parsetree = parser.parse().map_err(StoreError::ParseError)?;
        let validator = lore_parser::Validator::new();
        let ast = validator
            .validate(parsetree)
            .map_err(StoreError::ValidationError)?;
        self.add_tree(ast)
    }

    pub fn add_tree(&mut self, ast: lore_ast::Structure) -> Result<&mut Store, StoreError> {
        for attribute in ast.attributes {
            self.attributes
                .insert(attribute.name.to_uri(), attribute.clone());
        }

        for kind in ast.kinds {
            self.kinds.insert(kind.name.to_uri(), kind.clone());
        }

        for rel in ast.relations {
            match self.relations_by_subject.get_mut(&rel.subject.to_uri()) {
                None => {
                    self.relations_by_subject
                        .insert(rel.subject.to_uri(), vec![rel.clone()]);
                }
                Some(q) => {
                    q.push(rel.clone());
                }
            }
        }

        Ok(self)
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
                let mut store = Store::new();
                store.add_from_string($src).unwrap();
                let snapshot = format!(
                    r#"
input:
    {}

output:

{:#?}
"#,
                    $src, store
                );
                assert_snapshot!(snapshot)
            }
        };
    }

    test!(store_kind_with_uri_name, "kind spotify:kind:artist");
    test!(
        store_kind_with_aliased_name,
        r#"
        prefix dota:ontology:v2021/Hero as @Hero
        kind @Hero
        "#
    );

    test!(store_attr_with_uri_name, "attr spotify:field:Name");
    test!(
        store_attr_with_aliased_name,
        r#"
        prefix spotify:field:Name as @Name
        attr @Name
        "#
    );

    test!(
        store_rel,
        r#"
        prefix dota:ontology:v2021/Hero as @Hero
        prefix spotify:attr:Name as @Name
        prefix spotify:rel:hasOne as @hasOne

        rel @Hero @hasOne @Name
        "#
    );

    test!(
        store_multiple_items,
        r#"
            prefix dota:ontology:2022/Hero as @Hero
            prefix dota:ontology:2022/Name as @Name
            prefix lore:rel:v1/hasOne as @hasOne

            kind @Hero

            attr @Name

            rel @Hero @hasOne @Name
        "#
    );
}
