use crate::source_set::*;
use std::fmt::{Display, Error, Formatter};
use std::path::PathBuf;

///////////////////////////////////////////////////////////////////////////////////////////////////
///
/// An OCaml Module
///

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CamlModuleName(String);

impl CamlModuleName {
    pub fn is_empty(&self) -> bool {
        self.0 == ""
    }

    pub fn local_module() -> CamlModuleName {
        CamlModuleName("".to_string())
    }

    pub fn from_name(name: &lore_ast::Name) -> CamlModuleName {
        let mut parts = vec![];
        for part in name.to_string().replace("/", ":").split(":") {
            parts.push(part.to_string())
        }

        let mut name = parts.join("_").to_lowercase();

        if let Some(first) = name.get_mut(0..1) {
            first.make_ascii_uppercase();
        };

        CamlModuleName(name)
    }
}

impl Display for CamlModuleName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CamlModule {
    name: CamlModuleName,
    filename: PathBuf,
    structure: Vec<CamlValue>,
}

impl Into<Source> for CamlModule {
    fn into(self) -> Source {
        Source::new(self.filename.clone(), format!("{}", self))
    }
}

impl Display for CamlModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for item in &self.structure {
            write!(f, "{}", item)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl CamlModule {
    pub fn new(name: CamlModuleName) -> CamlModule {
        let filename = PathBuf::from(format!("{}.mli", name.to_string().to_lowercase()));
        CamlModule {
            name,
            filename,
            structure: vec![],
        }
    }

    pub fn with_structure(self, structure: Vec<CamlValue>) -> CamlModule {
        CamlModule { structure, ..self }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///
/// An OCaml Type
///

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CamlField {
    name: String,
    doc: Option<String>,
    type_: CamlType,
}

impl Display for CamlField {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(doc) = &self.doc {
            write!(f, "(* {} *)", doc.trim())?;
        }
        write!(f, "{}: {}", self.name, self.type_)
    }
}

impl CamlField {
    pub fn from_name(name: &lore_ast::Name, type_: CamlType) -> CamlField {
        let mut parts = vec![];
        for part in name.to_string().replace("/", ":").split(":") {
            parts.push(part.to_string())
        }
        let mut name = parts.join("_");
        if let Some(first) = name.get_mut(0..1) {
            first.make_ascii_lowercase();
        };
        CamlField {
            name,
            type_,
            doc: None,
        }
    }

    pub fn with_doc(self, doc: Option<String>) -> CamlField {
        CamlField { doc, ..self }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CamlRecord {
    fields: Vec<CamlField>,
}

impl Display for CamlRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut fields = self.fields.clone();
        fields.sort_by(|a, b| a.cmp(b));

        write!(f, "{{ {}", fields[0])?;
        for field in fields[1..].iter() {
            write!(f, "\n; {}", field)?;
        }
        write!(f, " }}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CamlVariant {
    Constructor(Vec<CamlType>),
    InlineRecord(CamlRecord),
}

impl Display for CamlVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            CamlVariant::InlineRecord(r) => {
                write!(f, " of {}", r)
            }

            CamlVariant::Constructor(cs) => {
                if !cs.is_empty() {
                    write!(f, " of ")?;
                    write!(f, "{}", cs[0])?;
                    for c in cs[1..].iter() {
                        write!(f, "* {}", c)?;
                    }
                };
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CamlType {
    Reference {
        module_path: CamlModuleName,
        type_name: String,
    },
    Abstract(String),
    Variant {
        name: String,
        variants: Vec<CamlVariant>,
    },
    Record {
        name: String,
        record: CamlRecord,
    },
}

impl CamlType {
    pub fn reference(module_path: CamlModuleName, type_name: String) -> CamlType {
        CamlType::Reference {
            module_path,
            type_name,
        }
    }

    pub fn abstract_type(name: String) -> CamlType {
        CamlType::Abstract(name)
    }

    pub fn record(name: String, fields: Vec<CamlField>) -> CamlType {
        CamlType::Record {
            name,
            record: CamlRecord { fields },
        }
    }
}

impl Display for CamlType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            CamlType::Reference {
                type_name,
                module_path,
            } if module_path.is_empty() => {
                write!(f, "{}", type_name)
            }
            CamlType::Reference {
                module_path,
                type_name,
            } => write!(f, "{}.{}", module_path, type_name),
            CamlType::Abstract(name) => write!(f, "type {}", name),
            CamlType::Record { name, record } => {
                write!(f, "type {} = {} \n", name, record)
            }
            CamlType::Variant { name, variants } => {
                write!(f, "type {} = \n", name)?;
                for v in variants {
                    write!(f, "| {}", v)?;
                }
                Ok(())
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///
/// An OCaml Function
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CamlFun {
    args: Vec<CamlType>,
    return_: CamlType,
}

impl CamlFun {
    pub fn new(args: Vec<CamlType>, return_: CamlType) -> CamlFun {
        CamlFun { args, return_ }
    }
}

impl Display for CamlFun {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for arg in &self.args {
            write!(f, "{} -> ", arg)?;
        }
        write!(f, "{}", self.return_)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///
/// An OCaml Binding
///

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CamlBinding {
    name: String,
    val: CamlFun,
}

impl CamlBinding {
    pub fn bind(name: lore_ast::Name, val: CamlFun) -> CamlBinding {
        let mut parts = vec![];
        for part in name.to_string().replace("/", ":").split(":") {
            parts.push(part.to_string())
        }
        let mut name = parts.join("_");
        if let Some(first) = name.get_mut(0..1) {
            first.make_ascii_lowercase();
        };
        CamlBinding { name, val }
    }
}

impl Display for CamlBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "val {} : {}", self.name, self.val)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
///
/// An OCaml Value
///

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CamlValueDesc {
    Module(CamlModule),
    Type(CamlType),
    Binding(CamlBinding),
}

impl Display for CamlValueDesc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            CamlValueDesc::Module(m) => write!(f, "{}", m),
            CamlValueDesc::Type(t) => write!(f, "{}", t),
            CamlValueDesc::Binding(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CamlValue {
    value: CamlValueDesc,
    doc: Option<String>,
}

impl CamlValue {
    pub fn binding(b: CamlBinding) -> CamlValue {
        CamlValue {
            value: CamlValueDesc::Binding(b),
            doc: None,
        }
    }

    pub fn new_type(t: CamlType) -> CamlValue {
        CamlValue {
            value: CamlValueDesc::Type(t),
            doc: None,
        }
    }

    pub fn with_doc(self, doc: Option<String>) -> CamlValue {
        CamlValue { doc, ..self }
    }
}

impl Display for CamlValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(doc) = &self.doc {
            writeln!(f, "(*")?;
            writeln!(f, "  {}", doc.trim())?;
            writeln!(f, "*)")?;
        }
        write!(f, "{}", self.value)
    }
}
