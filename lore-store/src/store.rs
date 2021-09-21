use lore_ast::*;
use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Store {
    pub relations: Vec<Relation>,

    pub attributes: Vec<Attribute>,

    pub kinds: Vec<Kind>,
}

#[derive(Error, Debug)]
pub enum StoreError {
    #[error(transparent)]
    ParseError(#[from] lore_parser::ParseError),

    #[error("Many validation errors oh no")]
    ValidationError(Vec<lore_parser::ValidationError>),

    #[error("Runtime error")]
    Runtime(String),
}

impl Store {
    pub fn new() -> Store {
        Store::default()
    }

    pub fn kinds(&self) -> &Vec<Kind> {
        &self.kinds
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
        self.relations = ast.relations.clone();
        self.kinds = ast.kinds.clone();
        self.attributes = ast.attributes;

        Ok(self)
    }

    pub fn add_kind(&mut self, kind: Kind) {
        self.kinds.push(kind)
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute)
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
                let result = store.add_from_string($src);
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

    test!(store_kind_with_uri_name, "kind spotify:kind:artist");
    test!(
        store_kind_with_aliased_name,
        r#"
        use spotify:kind:artist as Artist
        kind Artist
        "#
    );

    test!(store_attr_with_uri_name, "attr spotify:field:Name");
    test!(
        store_attr_with_aliased_name,
        r#"
        use spotify:field:Name as Name
        attr Name
        "#
    );

    test!(
        store_rel,
        r#"
        use spotify:kind:artist as Artist
        use spotify:attr:Name as Name
        use spotify:rel:hasOne as hasOne

        rel Artist hasOne Name
        "#
    );

    test!(
        store_multiple_items,
        r#"
            use spotify:kind:artist as Artist
            kind Artist
            attr spotify:field:play_count
            kind spotify:kind:Album
            use spotify:kind:song as Song
            kind Song

            rel Artist spotify:rel:isAuthorOf Song
            rel Song spotify:rel:isPerformedBy Artist
        "#
    );
}
