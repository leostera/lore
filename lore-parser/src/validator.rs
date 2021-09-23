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

        let mut local_namespace: Option<URI> = None;
        let mut relations: Vec<lore_ast::Relation> = vec![];
        let mut kinds: Vec<lore_ast::Kind> = vec![];
        let mut attributes: Vec<lore_ast::Attribute> = vec![];
        let mut aliases: HashMap<String, URI> = HashMap::new();

        for item in parsetree.items() {
            match item {
                StructureItem::Namespace { uri } => {
                    local_namespace.replace(uri.clone());
                }

                StructureItem::Alias { uri, prefix } => {
                    aliases.insert(prefix.clone().to_string(), uri.clone());
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
                if let Some(uri) = Validator::normalize_prefixed_uri(
                    &attribute.name,
                    &aliases,
                    &local_namespace,
                    &mut errors,
                ) {
                    attribute.name.set_uri(&uri);
                }
            }
        }

        for kind in &mut kinds {
            if kind.name.is_unresolved() {
                if let Some(uri) = Validator::normalize_prefixed_uri(
                    &kind.name,
                    &aliases,
                    &local_namespace,
                    &mut errors,
                ) {
                    kind.name.set_uri(&uri);
                }
            }
        }

        for relation in &mut relations {
            if relation.subject.is_unresolved() {
                if let Some(uri) = Validator::normalize_prefixed_uri(
                    &relation.subject,
                    &aliases,
                    &local_namespace,
                    &mut errors,
                ) {
                    relation.subject.set_uri(&uri);
                }
            }
            if relation.predicate.is_unresolved() {
                if let Some(uri) = Validator::normalize_prefixed_uri(
                    &relation.predicate,
                    &aliases,
                    &local_namespace,
                    &mut errors,
                ) {
                    relation.predicate.set_uri(&uri);
                }
            }
            if relation.object.is_unresolved() {
                if let Some(uri) = Validator::normalize_prefixed_uri(
                    &relation.object,
                    &aliases,
                    &local_namespace,
                    &mut errors,
                ) {
                    relation.object.set_uri(&uri);
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

    pub fn normalize_prefixed_uri(
        name: &lore_ast::Name,
        aliases: &HashMap<String, URI>,
        local_namespace: &Option<URI>,
        errors: &mut Vec<ValidationError>,
    ) -> Option<lore_ast::URI> {
        if let Some(alias) = &name.alias {
            if let Some(uri) = &local_namespace {
                Some(uri.join(alias))
            } else {
                errors.push(ValidationError::UnresolvedName(name.clone()));
                None
            }
        } else {
            for (prefix, expanded_uri) in aliases.into_iter() {
                if name.uri.has_prefix(prefix) {
                    return Some(name.uri.expand_prefix(prefix, expanded_uri));
                }
            }
            errors.push(ValidationError::UnresolvedName(name.clone()));
            None
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
                let mut parser = parser::Parser::for_string("$name", $src).unwrap();
                let validator = Validator::new();
                let parsetree = parser.parse().unwrap();
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
        " prefix spotify:artist:2Hkut4rAAyrQxRdof7FVJq as @Rush "
    );

    test!(validate_missing_alias_on_kind, "kind Band");

    test!(validate_missing_alias_on_attr, "attr Role");

    test!(
        validate_missing_alias_on_rel,
        r#"
        prefix spotify:kind:artist as @Artist
        prefix spotify:kind:song as @Song
        prefix spotify:rel:hasOne as @hasOne
        prefix spotify:rel:hasMany as @hasMany

        rel @Artist @hasOne @Name
        rel @Artist @isAuthorOf @Song
        rel @Band @hasMany @Song
        "#
    );

    test!(
        normalize_aliases_on_attr,
        r#"
        prefix spotify:attr as @spotifyAttributes
        attr @spotifyAttributes/name
        "#
    );
    test!(
        normalize_aliases_on_kind,
        r#"
        prefix spotify:kind:artist as @Artist
        kind @Artist
        "#
    );
    test!(
        normalize_aliases_on_rels,
        r#"
        prefix spotify:kind:artist as @Artist
        prefix spotify:attr:Name as @Name
        prefix spotify:rel:hasOne as @hasOne
        rel @Artist @hasOne @Name
        "#
    );
}
