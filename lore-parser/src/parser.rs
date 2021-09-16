use crate::lexer::Token;
use logos::Logos;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Uri(String);

#[derive(Clone, Debug)]
pub enum Literal {
    Number(u64),
    String(String),
    Uri(Uri),
}

#[derive(Clone, Debug)]
pub struct Field {
    name: Uri,
    value: Literal,
}

#[derive(Clone, Debug)]
pub enum StructureItem {
    Comment(String),

    Alias { uri: Uri, prefix: String },

    Annotation { name: Uri, value: Option<Literal> },

    Kind { name: Uri },

    Attribute { name: Uri, fields: Vec<Field> },
}

pub struct Structure(Vec<StructureItem>);

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Runtime error")]
    Runtime(String),
}

fn parse(text: &str) -> Result<Structure, ParseError> {
    let mut lex = Token::lexer(text);

    Ok(Structure(vec![]))
}
