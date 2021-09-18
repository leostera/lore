use crate::lexer::Token;
use crate::parsetree::*;
use logos::{Lexer, Logos};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("Expected a URI.")]
    ExpectedURI,

    #[error("The `use` syntax expects a URI.")]
    UseExpectsURI,

    #[error(
        "The `use` syntax should follow the format: use <uri> as <name>. Did you forget the 'as'?"
    )]
    UseIsMissingTheAsKeyword,

    #[error(r#"
        The `use` syntax should follow the format: use <uri> as <name>. Did you forget to specify a name?
    "#)]
    UseIsMissingTheAliasedName,

    #[error("")]
    UseExpectsTextPrefix,

    #[error("The `kind <name>` syntax is missing a name.")]
    KindIsMissingAName,

    #[error("We expected to find a Name (an Alias or a URI).")]
    NameIsInvalid,

    #[error("Runtime error")]
    Runtime(String),
}

pub fn parse(text: &str) -> Result<Structure, ParseError> {
    let mut lex = Token::lexer(text);
    parse_structure(&mut lex)
}

fn parse_structure(mut lex: &mut Lexer<Token>) -> Result<Structure, ParseError> {
    let mut items = vec![];
    while let Some(token) = lex.next() {
        let item = match token {
            Token::Use => parse_alias(&mut lex)?,
            Token::Kind => parse_kind(&mut lex)?,
            Token::Attribute => parse_attr(&mut lex)?,
            _ => continue,
        };
        items.push(item);
    }
    Ok(Structure(items))
}

fn parse_uri(lex: &mut Lexer<Token>) -> Result<URI, ParseError> {
    match lex.next() {
        Some(Token::URI(uri)) => Ok(URI(uri)),
        _ => Err(ParseError::ExpectedURI),
    }
}

fn parse_name(lex: &mut Lexer<Token>) -> Result<Name, ParseError> {
    match lex.next() {
        Some(Token::URI(uri)) => Ok(Name::URI(URI(uri))),
        Some(Token::Text(text)) => Ok(Name::Alias(text)),
        _ => Err(ParseError::NameIsInvalid),
    }
}

fn parse_alias(mut lex: &mut Lexer<Token>) -> Result<StructureItem, ParseError> {
    let uri = match parse_uri(&mut lex) {
        Ok(uri) => Ok(uri),
        _ => Err(ParseError::UseExpectsURI),
    }?;

    match lex.next() {
        Some(Token::As) => Ok(()),
        _ => Err(ParseError::UseIsMissingTheAsKeyword),
    }?;

    let prefix = match lex.next() {
        Some(Token::Text(prefix)) => Ok(prefix),
        _ => Err(ParseError::UseIsMissingTheAliasedName),
    }?;

    Ok(StructureItem::Alias { uri, prefix })
}

fn parse_kind(mut lex: &mut Lexer<Token>) -> Result<StructureItem, ParseError> {
    let name = match parse_name(&mut lex) {
        Ok(name) => Ok(name),
        _ => Err(ParseError::KindIsMissingAName),
    }?;

    Ok(StructureItem::Kind { name })
}

fn parse_attr(mut lex: &mut Lexer<Token>) -> Result<StructureItem, ParseError> {
    let name = match parse_name(&mut lex) {
        Ok(name) => Ok(name),
        _ => Err(ParseError::KindIsMissingAName),
    }?;

    let fields = vec![];

    Ok(StructureItem::Attribute { name, fields })
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::*;

    macro_rules! test {
        ($name:ident, $src:expr) => {
            #[test]
            fn $name() {
                let result = parse($src);
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
        parse_alias,
        r#" use spotify:artist:2Hkut4rAAyrQxRdof7FVJq as Rush "#
    );

    test!(
        parse_alias_with_something_that_isnt_a_uri,
        r#" use 2Hkut4rAAyrQxRdof7FVJq as Rush "#
    );

    test!(parse_alias_with_a_missing_uri, r#" use as Rush "#);

    test!(
        parse_alias_with_uri_but_missing_keyword_as,
        r#" use spotify:hello Rush "#
    );

    test!(
        parse_alias_with_uri_but_missing_the_aliased_name,
        r#" use spotify:hello as"#
    );

    test!(parse_kind_with_uri_name, r#"kind spotify:kind:artist"#);

    test!(parse_kind_with_aliased_name, r#"kind Hello"#);

    test!(parse_kind_with_missing_name, r#"kind"#);

    test!(parse_attr_with_uri_name, r#"attr spotify:field:Name"#);

    test!(parse_attr_without_aliased_name, r#"attr Name"#);

    test!(
        parse_multiple_items,
        r#"
            use spotify:kind:artist as Artist
            kind Artist
            attr Name
            attr spotify:field:play_count
            kind spotify:kind:Album
            use spotify:kind:song as Song
            kind Song
        "#
    );
}
