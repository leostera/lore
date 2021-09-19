use crate::parsetree::*;
use lore_ast::URI;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Validator {}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ValidationError {
    #[error("This name cannot be resolved, did you forget to add a `use` alias?")]
    UnresolvedName(lore_ast::Name),

    #[error("Runtime error")]
    Runtime(String),
}

impl Validator {
    pub fn new() -> Validator {
        Validator::default()
    }

    pub fn validate(
        &self,
        parsetree: Structure,
    ) -> Result<lore_ast::Structure, Vec<ValidationError>> {
        let mut errors = vec![];

        let mut relations: Vec<lore_ast::Relation> = vec![];
        let mut kinds: Vec<lore_ast::Kind> = vec![];
        let mut attributes: Vec<lore_ast::Attribute> = vec![];
        let mut aliases: HashMap<String, URI> = HashMap::new();

        for item in parsetree.items() {
            match item {
                StructureItem::Alias { uri, prefix } => {
                    aliases.insert(prefix.clone(), uri.clone());
                }

                StructureItem::Kind { name, .. } => {
                    let name: lore_ast::Name = name.into();
                    kinds.push(lore_ast::Kind { name });
                }

                StructureItem::Attribute { name, .. } => {
                    let name: lore_ast::Name = name.into();
                    attributes.push(lore_ast::Attribute { name });
                }

                StructureItem::Relation {
                    subject,
                    predicate,
                    object,
                } => relations.push(lore_ast::Relation {
                    subject: subject.into(),
                    predicate: predicate.into(),
                    object: object.into(),
                }),

                StructureItem::Comment(_) => (),
            }
        }

        for attribute in &mut attributes {
            if attribute.name.is_unresolved() {
                if let Some(alias) = &attribute.name.alias {
                    if let Some(uri) = aliases.get(alias) {
                        attribute.name.set_uri(uri);
                    } else {
                        errors.push(ValidationError::UnresolvedName(attribute.name.clone()));
                    }
                }
            }
        }

        for kind in &mut kinds {
            if kind.name.is_unresolved() {
                if let Some(alias) = &kind.name.alias {
                    if let Some(uri) = aliases.get(alias) {
                        kind.name.set_uri(uri);
                    } else {
                        errors.push(ValidationError::UnresolvedName(kind.name.clone()));
                    }
                }
            }
        }

        for relation in &mut relations {
            if relation.subject.is_unresolved() {
                if let Some(alias) = &relation.subject.alias {
                    if let Some(uri) = aliases.get(alias) {
                        relation.subject.set_uri(uri);
                    } else {
                        errors.push(ValidationError::UnresolvedName(relation.subject.clone()));
                    }
                }
            }
            if relation.predicate.is_unresolved() {
                if let Some(alias) = &relation.predicate.alias {
                    if let Some(uri) = aliases.get(alias) {
                        relation.predicate.set_uri(uri);
                    } else {
                        errors.push(ValidationError::UnresolvedName(relation.predicate.clone()));
                    }
                }
            }
            if relation.object.is_unresolved() {
                if let Some(alias) = &relation.object.alias {
                    if let Some(uri) = aliases.get(alias) {
                        relation.object.set_uri(uri);
                    } else {
                        errors.push(ValidationError::UnresolvedName(relation.object.clone()));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(lore_ast::Structure {
                kinds,
                attributes,
                relations,
            })
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use insta::*;

    macro_rules! test {
        ($name:ident, $src:expr) => {
            #[test]
            fn $name() {
                let validator = Validator::new();
                let parsetree = parser::parse($src).unwrap();
                let result = validator.validate(parsetree);
                let snapshot = format!(
                    r#"
input:
    {}

output:

{:#?}
"#,
                    $src, result
                );
                assert_snapshot!(snapshot)
            }
        };
    }

    test!(
        validate_aliasing,
        " use spotify:artist:2Hkut4rAAyrQxRdof7FVJq as Rush "
    );

    test!(validate_missing_alias_on_kind, "kind Band");

    test!(validate_missing_alias_on_attr, "attr Role");

    test!(
        validate_missing_alias_on_rel,
        r#"
        use spotify:kind:artist as Artist
        use spotify:kind:song as Song
        use spotify:rel:hasOne as hasOne
        use spotify:rel:hasMany as hasMany

        rel Artist hasOne Name
        rel Artist isAuthorOf Song
        rel Band hasMany Song
        "#
    );

    test!(
        normalize_aliases_on_attr,
        r#"
        use spotify:attr:name as Name
        attr Name
        "#
    );
    test!(
        normalize_aliases_on_kind,
        r#"
        use spotify:kind:artist as Artist
        kind Artist
        "#
    );
    test!(
        normalize_aliases_on_rels,
        r#"
        use spotify:kind:artist as Artist
        use spotify:attr:Name as Name
        use spotify:rel:hasOne as hasOne
        rel Artist hasOne Name
        "#
    );
}
