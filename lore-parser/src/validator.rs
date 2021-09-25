use crate::parsetree::*;
use lore_ast::URI;
use miette::Diagnostic;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Validator {
    local_namespace: Option<URI>,
    relations: Vec<lore_ast::Relation>,
    kinds: Vec<lore_ast::Kind>,
    attributes: Vec<lore_ast::Attribute>,
    aliases: HashMap<String, URI>,
    unresolved_names: Vec<lore_ast::Name>,
}

fn format_names(names: Vec<lore_ast::Name>) -> String {
    let mut strs = vec![];
    for name in names {
        if let Some(str) = name.alias {
            strs.push(format!("* {}", str))
        } else {
            strs.push(format!("* {}", name.to_string()))
        }
    }
    strs.join("\n").to_string()
}

#[derive(Error, Debug, Diagnostic)]
#[diagnostic(code(lore::validator::semantic))]
pub enum SemanticError {
    #[error("The follow names cannot be resolved: \n{}\nDid you forget to add a `prefix` alias or a `using` namespace?", format_names(.0.to_vec()))]
    UnresolvedNames(Vec<lore_ast::Name>),
}

#[derive(Error, Debug, Diagnostic)]
#[error("Validation error on file {filename:?}")]
#[diagnostic(code(lore::validator), url(docsrs))]
pub struct ValidationError {
    filename: PathBuf,

    #[source]
    error: SemanticError,
}

impl Validator {
    pub fn new() -> Validator {
        Validator::default()
    }

    pub fn validate(
        mut self,
        parsetree: Structure,
    ) -> Result<lore_ast::Structure, ValidationError> {
        for item in parsetree.items() {
            match item {
                StructureItem::Namespace { uri } => {
                    self.local_namespace.replace(uri.clone());
                }

                StructureItem::Alias { uri, prefix } => {
                    self.aliases.insert(prefix.clone().to_string(), uri.clone());
                }
                _ => continue,
            }
        }

        for item in parsetree.items() {
            match item {
                StructureItem::Kind { name, fields } => {
                    let name = self.normalize_name(name);
                    let fields = self.normalize_fields(&fields);
                    self.kinds.push(lore_ast::Kind { name, fields });
                }

                StructureItem::Attribute { name, fields } => {
                    let name = self.normalize_name(name);
                    let fields = self.normalize_fields(&fields);
                    self.attributes.push(lore_ast::Attribute { name, fields });
                }

                StructureItem::Relation {
                    subject,
                    predicate,
                    object,
                    fields,
                } => {
                    let subject = self.normalize_name(subject);
                    let predicate = self.normalize_name(predicate);
                    let object = self.normalize_name(object);
                    let fields = self.normalize_fields(&fields);
                    self.relations.push(lore_ast::Relation {
                        subject,
                        predicate,
                        object,
                        fields,
                    })
                }

                _ => (),
            }
        }

        if self.unresolved_names.is_empty() {
            Ok(lore_ast::Structure {
                kinds: self.kinds,
                attributes: self.attributes,
                relations: self.relations,
            })
        } else {
            Err(ValidationError {
                filename: parsetree.filename().clone(),
                error: SemanticError::UnresolvedNames(self.unresolved_names.clone()),
            })
        }
    }

    pub fn normalize_name(&mut self, name: &Name) -> lore_ast::Name {
        let mut name: lore_ast::Name = name.into();
        let alias = name.alias.clone();
        match alias {
            Some(alias) => {
                if let Some(uri) = &self.local_namespace {
                    name.set_uri(&uri.join(&alias));
                } else {
                    self.unresolved_names.push(name.clone());
                }
            }
            None => {
                if name.is_unresolved() {
                    for (prefix, expanded_uri) in (&self.aliases).into_iter() {
                        if name.uri.has_prefix(&prefix) {
                            name.set_uri(&name.uri.expand_prefix(&prefix, &expanded_uri));
                            return name;
                        }
                    }
                    self.unresolved_names.push(name.clone());
                }
            }
        }
        name
    }

    pub fn normalize_literal(&mut self, lit: &Literal) -> lore_ast::Literal {
        match lit {
            Literal::Number(n) => lore_ast::Literal::Number(*n),
            Literal::String(s) => lore_ast::Literal::String(s.to_string()),
            Literal::Name(n) => lore_ast::Literal::Name(self.normalize_name(n)),
        }
    }

    pub fn normalize_fields(&mut self, fields: &[Field]) -> Vec<lore_ast::Field> {
        let mut ast_fields = vec![];

        for field in fields {
            let field = lore_ast::Field {
                name: self.normalize_name(&field.name),
                value: self.normalize_literal(&field.value),
            };
            ast_fields.push(field);
        }

        ast_fields
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
    test!(
        normalize_aliases_on_meta_attrs,
        r#"
        using hello:world
        attr name {
            wat :yes
        }
        "#
    );

    test!(
        normalize_meta_attrs_on_kind,
        r#"
        using hello:world
        kind person {
            lore:v1/doc """hello"""
        }
        attr name {
            lore:v1/doc 1234
        }
        rel person has name {
            lore:v1/doc :no-doc
        }
        "#
    );
}
