#[derive(Clone, Debug, Default, PartialEq)]
pub struct URI(pub String);

#[derive(Clone, Debug, PartialEq)]
pub enum Name {
    URI(URI),
    Alias(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Number(u64),
    String(String),
    URI(URI),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    name: URI,
    value: Literal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StructureItem {
    Comment(String),

    Alias { uri: URI, prefix: String },

    Directive { name: Name, value: Option<Literal> },

    Kind { name: Name },

    Attribute { name: Name, fields: Vec<Field> },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Structure(pub Vec<StructureItem>);

impl Structure {
    pub fn items(&self) -> &Vec<StructureItem> {
        &self.0
    }
}
