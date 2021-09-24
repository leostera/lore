use lore_ast::URI;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pos {
    file: String,
    line: u32,
    col: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Name {
    URI(URI),
    Alias(String),
}

impl Into<lore_ast::Name> for &Name {
    fn into(self) -> lore_ast::Name {
        match self {
            Name::URI(uri) => lore_ast::Name::of_uri(&uri),
            Name::Alias(alias) => lore_ast::Name::unresolved_alias(alias),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Number(u64),
    String(String),
    Name(Name),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: Name,
    pub value: Literal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StructureItem {
    Namespace {
        uri: URI,
    },

    Comment(String),

    Alias {
        uri: URI,
        prefix: URI,
    },

    Kind {
        name: Name,
        fields: Vec<Field>,
    },

    Attribute {
        name: Name,
        fields: Vec<Field>,
    },

    Relation {
        subject: Name,
        predicate: Name,
        object: Name,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Structure {
    items: Vec<StructureItem>,
}

impl Structure {
    pub fn of_items(items: Vec<StructureItem>) -> Structure {
        Structure { items }
    }

    pub fn items(&self) -> &Vec<StructureItem> {
        &self.items
    }
}
