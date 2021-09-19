use crate::emitter_error::*;
use graphql_parser::schema::*;
use graphql_parser::Pos;

#[derive(Default)]
pub struct GraphQLEmitter {}

// impl LoreEmitter<Document> for GraphQLEmitter {
impl GraphQLEmitter {
    pub fn new() -> GraphQLEmitter {
        GraphQLEmitter::default()
    }

    pub fn translate(
        &self,
        store: &lore_store::Store,
    ) -> Result<Document<'static, String>, EmitterError> {
        let mut definitions = vec![];

        for kind in store.kinds() {
            for typedef in self.kind_to_types(kind)? {
                definitions.push(typedef);
            }
        }

        Ok(Document { definitions })
    }

    fn name_to_type_name(&self, name: &lore_ast::Name) -> String {
        name.to_string().replace(":", "_").replace("/", "__")
    }

    fn name_to_input_name(&self, name: &lore_ast::Name) -> String {
        format!("{}__Input", self.name_to_type_name(name))
    }

    fn kind_to_types(
        &self,
        kind: &lore_ast::Kind,
    ) -> Result<Vec<Definition<'static, String>>, EmitterError> {
        let position = Pos { line: 0, column: 0 };
        Ok(vec![
            Definition::TypeDefinition(TypeDefinition::Object(ObjectType {
                name: self.name_to_type_name(&kind.name),
                position,
                implements_interfaces: vec![],
                description: None,
                directives: vec![],
                fields: vec![],
            })),
            Definition::TypeDefinition(TypeDefinition::InputObject(InputObjectType {
                name: self.name_to_input_name(&kind.name),
                position,
                description: None,
                directives: vec![],
                fields: vec![],
            })),
        ])
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
                let emitter = GraphQLEmitter::new();
                let document = emitter.translate(&store).unwrap();
                let snapshot = format!(
                    r#"
input:
    {}

output:

{}
"#,
                    $src, document
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
