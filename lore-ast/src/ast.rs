use crate::uri::*;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
    pub alias: Option<String>,
    pub uri: URI,
}

impl ToString for Name {
    fn to_string(&self) -> String {
        self.uri.to_string()
    }
}

impl Name {
    pub fn of_uri(uri: &URI) -> Name {
        Name {
            uri: uri.clone(),
            alias: None,
        }
    }

    pub fn to_uri(&self) -> URI {
        self.uri.clone()
    }

    pub fn set_uri(&mut self, uri: &URI) {
        self.uri = uri.clone();
    }

    pub fn is_unresolved(&self) -> bool {
        self.uri == URI::unresolved() || self.uri.is_prefixed()
    }

    pub fn unresolved_alias(alias: &str) -> Name {
        Name {
            uri: URI::unresolved(),
            alias: Some(alias.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    Number(u64),
    String(String),
    Name(Name),
}

impl ToString for Literal {
    fn to_string(&self) -> String {
        match self {
            Literal::Number(n) => n.to_string(),
            Literal::String(s) => s.to_string(),
            Literal::Name(n) => n.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field {
    pub name: Name,
    pub value: Literal,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attribute {
    pub name: Name,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Kind {
    pub name: Name,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Relation {
    pub subject: Name,
    pub predicate: Name,
    pub object: Name,
    pub fields: Vec<Field>,
}

/*
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum StructureItem {
    Relation(Relation),
    Kind(Kind),
    Attribute(Attribute),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Structure {
    pub items: Vec<StructureItem>,
}
*/

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Structure {
    pub kinds: Vec<Kind>,

    pub attributes: Vec<Attribute>,

    pub relations: Vec<Relation>,
}
