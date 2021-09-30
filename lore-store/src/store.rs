use crate::quads::ToQuads;
use lore_ast::*;
use miette::Diagnostic;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Diagnostic, Error, Debug)]
#[diagnostic(code(lore::store), url(docsrs))]
pub enum StoreError {
    #[error(transparent)]
    ParseError(#[from] lore_parser::ParseError),

    #[error(transparent)]
    ValidationError(#[from] lore_parser::ValidationError),

    #[error(transparent)]
    QueryError(#[from] oxigraph::sparql::EvaluationError),

    #[error("Runtime error")]
    Runtime(String),
}

#[derive(Clone, Default)]
pub struct Store {
    pub graph: oxigraph::MemoryStore,

    pub relations_by_subject: HashMap<URI, Vec<Relation>>,

    pub attributes: HashMap<URI, Attribute>,

    pub kinds: HashMap<URI, Kind>,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "Store {{")?;

        writeln!(f, "  kinds: {{")?;
        let mut kinds: Vec<Kind> = self.kinds.values().cloned().collect();
        kinds.sort();
        for k in kinds {
            writeln!(f, "  {:#?}", k)?;
        }
        writeln!(f, "  }}")?;

        writeln!(f, "  attributes: {{")?;
        let mut attributes: Vec<Attribute> = self.attributes.values().cloned().collect();
        attributes.sort();
        for a in attributes {
            writeln!(f, "    {:#?}", a)?;
        }
        writeln!(f, "  }}")?;

        writeln!(f, "  relations: {{")?;
        let mut relations: Vec<(URI, Vec<Relation>)> =
            self.relations_by_subject.clone().into_iter().collect();
        relations.sort();
        for rel in relations {
            writeln!(f, "    {:#?}", rel)?;
        }
        writeln!(f, "  }}")?;

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

            for q in attribute.to_quads() {
                self.graph.insert(q);
            }
        }

        for kind in ast.kinds {
            self.kinds.insert(kind.name.to_uri(), kind.clone());

            for q in kind.to_quads() {
                self.graph.insert(q);
            }
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

            if let Some(rels) = self.relations_by_subject.get_mut(&rel.subject.to_uri()) {
                for rel in rels {
                    for q in rel.to_quads() {
                        self.graph.insert(q);
                    }
                }
            }
        }

        Ok(self)
    }

    pub fn query(&self, query: &str) -> Result<Vec<Vec<String>>, StoreError> {
        let query = format!(
            r#"

PREFIX lore: <https://lore-lang.org/v1/>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX owl: <http://www.w3.org/2002/07/owl#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

{}
        "#,
            query
        );

        use oxigraph::sparql::QueryResults;
        if let QueryResults::Solutions(solutions) =
            self.graph.query(&query).map_err(StoreError::QueryError)?
        {
            let mut results = vec![];
            for solution in solutions {
                match solution {
                    Err(e) => {
                        return Err(StoreError::QueryError(e));
                    }
                    Ok(s) => {
                        let mut vars = vec![];
                        for (var, term) in s.iter() {
                            vars.push(format!("{}: {}", var, term));
                        }
                        results.push(vars);
                    }
                }
            }
            Ok(results)
        } else {
            Ok(vec![])
        }
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
