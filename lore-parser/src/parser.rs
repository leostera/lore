use crate::lexer::Token;
use crate::parsetree::*;
use logos::{Lexer, Logos};
use lore_ast::URI;
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

    #[error("We expected to find a Name (an Alias or a URI) but found something else instead.")]
    NameIsInvalid,

    #[error("We expected to find a Name, did you forget it?")]
    NameIsMissing,

    #[error("The `rel` syntax should follow the format: rel <subject> <predicate> <object>. All 3 must be URIs or aliased names.")]
    RelationExpectedSubjectToBeName,

    #[error("The `rel` syntax should follow the format: rel <subject> <predicate> <object>. All 3 must be URIs or aliased names.")]
    RelationExpectedPredicateToBeName,

    #[error("The `rel` syntax should follow the format: rel <subject> <predicate> <object>. All 3 must be URIs or aliased names.")]
    RelationExpectedObjectToBeName,

    #[error("We expected to find an Alias, a Kind, an Attribute, or a Relation.")]
    ExpectedTopLevelItem,

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
            Token::Use => parse_alias(&mut lex),
            Token::Kind => parse_kind(&mut lex),
            Token::Attribute => parse_attr(&mut lex),
            Token::Relation => parse_rel(&mut lex),
            _ => Err(ParseError::ExpectedTopLevelItem),
        }?;
        items.push(item);
    }
    Ok(Structure::of_items(items))
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
        None => Err(ParseError::NameIsMissing),
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

    Ok(StructureItem::Attribute { name })
}

fn parse_rel(mut lex: &mut Lexer<Token>) -> Result<StructureItem, ParseError> {
    let subject = match parse_name(&mut lex) {
        Ok(name) => Ok(name),
        _ => Err(ParseError::RelationExpectedSubjectToBeName),
    }?;

    let predicate = match parse_name(&mut lex) {
        Ok(name) => Ok(name),
        _ => Err(ParseError::RelationExpectedPredicateToBeName),
    }?;

    let object = match parse_name(&mut lex) {
        Ok(name) => Ok(name),
        _ => Err(ParseError::RelationExpectedObjectToBeName),
    }?;

    Ok(StructureItem::Relation {
        subject,
        predicate,
        object,
    })
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
        " use spotify:artist:2Hkut4rAAyrQxRdof7FVJq as Rush "
    );

    test!(
        parse_alias_with_something_that_isnt_a_uri,
        " use 2Hkut4rAAyrQxRdof7FVJq as Rush "
    );

    test!(parse_alias_with_a_missing_uri, " use as Rush ");

    test!(
        parse_alias_with_uri_but_missing_keyword_as,
        " use spotify:hello Rush "
    );

    test!(
        parse_alias_with_uri_but_missing_the_aliased_name,
        " use spotify:hello as"
    );

    test!(parse_kind_with_uri_name, "kind spotify:kind:artist");

    test!(parse_kind_with_aliased_name, "kind Hello");

    test!(parse_kind_with_missing_name, "kind");

    test!(parse_attr_with_uri_name, "attr spotify:field:Name");

    test!(parse_attr_with_aliased_name, "attr Name");

    test!(parse_rel_incomplete_with_1_part_aliased, "rel Artist");

    test!(parse_rel_incomplete_with_1_part, "rel spotify:kinds/Artist");

    test!(parse_rel_incomplete_with_2_parts_aliased, "rel Artist has");

    test!(
        parse_rel_incomplete_with_2_parts,
        "rel spotify:kinds/Artist spotify:rels/has"
    );

    test!(
        parse_rel_complete_with_3_parts_aliased,
        "rel Artist has Name"
    );

    test!(
        parse_rel_complete_with_3_parts,
        "rel spotify:kinds/Artist spotify:rels/has spotify:attrs/Name"
    );

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
