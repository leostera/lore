use lore_ast::*;
use oxigraph::model::*;

pub trait ToQuads {
    fn to_quads(&self) -> Vec<Quad> {
        vec![]
    }
}

impl ToQuads for Kind {
    fn to_quads(&self) -> Vec<Quad> {
        let this = NamedNode::new(self.name.to_string()).unwrap();

        let owl_is_class = Quad::new(
            this.clone(),
            NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            NamedNode::new("http://www.w3.org/2002/07/owl#Class").unwrap(),
            None,
        );

        let lore_is_kind = Quad::new(
            this,
            NamedNode::new("https://lore-lang.org/v1/type").unwrap(),
            NamedNode::new("https://lore-lang.org/v1/Kind").unwrap(),
            None,
        );

        vec![owl_is_class, lore_is_kind]
    }
}

impl ToQuads for Attribute {
    fn to_quads(&self) -> Vec<Quad> {
        let this = NamedNode::new(self.name.to_string()).unwrap();

        let owl_is_object_property = Quad::new(
            this.clone(),
            NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap(),
            NamedNode::new("http://www.w3.org/2002/07/owl#ObjectProperty").unwrap(),
            None,
        );

        let lore_is_attribute = Quad::new(
            this,
            NamedNode::new("https://lore-lang.org/v1/type").unwrap(),
            NamedNode::new("https://lore-lang.org/v1/Attribute").unwrap(),
            None,
        );

        vec![owl_is_object_property, lore_is_attribute]
    }
}

impl ToQuads for Relation {
    fn to_quads(&self) -> Vec<Quad> {
        let this = NamedNode::new(self.predicate.to_string()).unwrap();

        let rdf_domain = Quad::new(
            this.clone(),
            NamedNode::new("http://www.w3.org/2000/01/rdf-schema#domain").unwrap(),
            NamedNode::new(self.subject.to_string()).unwrap(),
            None,
        );

        let rdf_range = Quad::new(
            this,
            NamedNode::new("http://www.w3.org/2000/01/rdf-schema#range").unwrap(),
            NamedNode::new(self.object.to_string()).unwrap(),
            None,
        );

        vec![rdf_domain, rdf_range]
    }
}
