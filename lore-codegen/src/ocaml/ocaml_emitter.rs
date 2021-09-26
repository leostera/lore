use crate::emitter_error::*;
use crate::ocaml::ocaml_ast::*;
use crate::source_set::*;

#[derive(Default)]
pub struct OCamlEmitter {}

// impl LoreEmitter<Document> for OCamlEmitter {
impl OCamlEmitter {
    pub fn new() -> OCamlEmitter {
        OCamlEmitter::default()
    }

    pub fn translate(&self, store: &lore_store::Store) -> Result<SourceSet, EmitterError> {
        let mut sources = vec![];

        for attribute in store.attributes() {
            let mut fields = vec![];
            let subject = attribute.name.to_uri();
            if let Some(rels) = store.relations_by_subject.get(&subject) {
                for rel in rels {
                    let module_name = CamlModuleName::from_name(&rel.object);
                    let type_ref = CamlType::reference(module_name, "t".to_string());
                    let field = CamlField::from_name(&rel.predicate, type_ref);
                    fields.push(field);
                }
            };

            let type_name = "t".to_string();
            let main_type = if fields.is_empty() {
                CamlType::abstract_type(type_name)
            } else {
                CamlType::record(type_name, fields)
            };

            let module = CamlModule::new(CamlModuleName::from_name(&attribute.name))
                .with_structure(vec![CamlValue::new_type(main_type)]);
            sources.push(module.into())
        }

        for kind in store.kinds() {
            let mut doc_string = None;
            let mut fields = vec![];

            let mut accessors = vec![];

            for field in &kind.fields {
                if field.name.uri == lore_ast::URI::from_string("lore:v1/doc/en".to_string()) {
                    doc_string = Some(field.value.to_string())
                };
            }

            let subject = kind.name.to_uri();
            if let Some(rels) = store.relations_by_subject.get(&subject) {
                for rel in rels {
                    let module_name = CamlModuleName::from_name(&rel.object);
                    let type_ref = CamlType::reference(module_name, "t".to_string());

                    let doc = rel
                        .fields
                        .iter()
                        .find(|f| {
                            f.name.uri == lore_ast::URI::from_string("lore:v1/doc/en".to_string())
                        })
                        .map(|f| f.value.to_string());

                    let accessor = CamlValue::binding(CamlBinding::bind(
                        rel.predicate.clone(),
                        CamlFun::new(
                            vec![CamlType::reference(
                                CamlModuleName::local_module(),
                                "t".to_string(),
                            )],
                            type_ref.clone(),
                        ),
                    ))
                    .with_doc(doc);

                    accessors.push(accessor);
                }
            };

            let type_name = "t".to_string();
            let main_type = if fields.is_empty() {
                CamlType::abstract_type(type_name)
            } else {
                CamlType::record(type_name, fields)
            };

            let module = CamlModule::new(CamlModuleName::from_name(&kind.name)).with_structure(
                vec![
                    vec![CamlValue::new_type(main_type).with_doc(doc_string)],
                    accessors,
                ]
                .concat(),
            );
            sources.push(module.into())
        }

        Ok(SourceSet::from_sources(sources))
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

{:#?}
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

prefix lore:rel:v1 as @lore

using dota:ontology:2022

kind Hero

attr Name

rel Hero @lore/hasOne Name

        "#
    );
}
