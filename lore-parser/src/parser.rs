use crate::lexer::Token;
use crate::parsetree::*;
use logos::{Lexer, Logos};
use lore_ast::URI;
use std::path::PathBuf;
use thiserror::Error;

struct PeekableLexer<'source> {
    lexer: Lexer<'source, Token>,
    peeked: Option<Option<Token>>,
}

impl<'source> PeekableLexer<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
            peeked: None,
        }
    }

    fn peek(&mut self) -> &Option<Token> {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next());
        }
        self.peeked.as_ref().unwrap()
    }

    fn slice(&self) -> String {
        self.lexer.slice().to_string()
    }
}

impl<'source> Iterator for PeekableLexer<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if let Some(peeked) = self.peeked.take() {
            peeked
        } else {
            self.lexer.next()
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Expected a URI.")]
    ExpectedURI(Option<Token>),

    #[error("The `use` syntax expects a URI.")]
    UseExpectsURI,

    #[error(
        "The `prefix` syntax should follow the format: prefix <uri> as @<name>. Did you forget the 'as'?"
    )]
    PrefixIsMissingTheAsKeyword,

    #[error(r#"
        The `prefix` syntax should follow the format: prefix <uri> as @<name>. Did you forget to specify a name?
    "#)]
    PrefixIsMissingTheAliasedName,

    #[error(r#"
        The `prefix` syntax should follow the format: prefix <uri> as @<name>. Did you forget the @ before the prefix name?
    "#)]
    PrefixShouldBeginWithAnAt,

    #[error("")]
    UseExpectsTextPrefix,

    #[error("The `kind <name>` syntax is missing a name.")]
    KindIsMissingAName,

    #[error("The `attr <name>` syntax is missing a name.")]
    AttributeIsMissingAName,

    #[error("We expected to find a Name (an Alias or a URI) but found something else instead.")]
    NameIsInvalid(Option<Token>),

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

    #[error("Did you forget to close this block with a `}}` ? ")]
    IncompleteFieldBlock,

    #[error("Every field must have a URI on the left side.")]
    FieldExpectedURI,

    #[error("Every field must have a URI, a String, or a Number on the right side.")]
    FieldExpectedLiteral(Option<Token>),

    #[error("This literal is wrong I guess")]
    InvalidLiteral(Option<Token>),

    #[error(transparent)]
    FileError(#[from] std::io::Error),

    #[error("Runtime error")]
    Runtime(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parser {
    file: PathBuf,
    source: String,
}

impl Parser {
    pub fn for_file(file: PathBuf) -> Result<Parser, ParseError> {
        let source = std::fs::read_to_string(&file).map_err(ParseError::FileError)?;
        Ok(Parser { file, source })
    }

    pub fn for_string(filename: &str, source: &str) -> Result<Parser, ParseError> {
        Ok(Parser {
            file: PathBuf::from(filename),
            source: source.to_string(),
        })
    }

    pub fn parse(&mut self) -> Result<Structure, ParseError> {
        let mut lexer = PeekableLexer::new(&self.source);
        Parser::parse_structure(&mut lexer)
    }

    fn parse_structure(mut lex: &mut PeekableLexer) -> Result<Structure, ParseError> {
        let mut items = vec![];
        while let Some(token) = lex.next() {
            let item = Parser::parse_structure_item(&mut lex, token)?;
            items.push(item);
        }
        Ok(Structure::of_items(items))
    }

    fn parse_structure_item(
        mut lex: &mut PeekableLexer,
        token: Token,
    ) -> Result<StructureItem, ParseError> {
        match token {
            Token::Using => Parser::parse_using(&mut lex),
            Token::Prefix => Parser::parse_prefix(&mut lex),
            Token::Kind => Parser::parse_kind(&mut lex),
            Token::Attribute => Parser::parse_attr(&mut lex),
            Token::Relation => Parser::parse_rel(&mut lex),
            Token::Comment(_) => Parser::parse_comment(&mut lex),
            _ => Err(ParseError::ExpectedTopLevelItem),
        }
    }

    fn parse_uri(lex: &mut PeekableLexer) -> Result<URI, ParseError> {
        match lex.next() {
            Some(Token::URI(uri)) => Ok(URI::from_string(uri)),
            token => Err(ParseError::ExpectedURI(token)),
        }
    }

    fn parse_name(lex: &mut PeekableLexer) -> Result<Name, ParseError> {
        match lex.next() {
            Some(Token::URI(uri)) => Ok(Name::URI(URI::from_string(uri))),
            Some(Token::Text(text)) => Ok(Name::Alias(text)),
            None => Err(ParseError::NameIsMissing),
            t => Err(ParseError::NameIsInvalid(t)),
        }
    }

    fn parse_comment(lex: &mut PeekableLexer) -> Result<StructureItem, ParseError> {
        Ok(StructureItem::Comment(lex.slice()))
    }

    fn parse_prefix(mut lex: &mut PeekableLexer) -> Result<StructureItem, ParseError> {
        let uri = match Parser::parse_uri(&mut lex) {
            Ok(uri) => Ok(uri),
            _ => Err(ParseError::UseExpectsURI),
        }?;

        match lex.next() {
            Some(Token::As) => Ok(()),
            _ => Err(ParseError::PrefixIsMissingTheAsKeyword),
        }?;

        let prefix = match Parser::parse_uri(&mut lex) {
            Ok(uri) => Ok(uri),
            _ => Err(ParseError::PrefixIsMissingTheAliasedName),
        }?;

        Ok(StructureItem::Alias { uri, prefix })
    }

    fn parse_using(mut lex: &mut PeekableLexer) -> Result<StructureItem, ParseError> {
        let uri = match Parser::parse_uri(&mut lex) {
            Ok(uri) => Ok(uri),
            _ => Err(ParseError::UseExpectsURI),
        }?;

        Ok(StructureItem::Namespace { uri })
    }

    fn parse_kind(mut lex: &mut PeekableLexer) -> Result<StructureItem, ParseError> {
        let name = match Parser::parse_name(&mut lex) {
            Ok(name) => Ok(name),
            _ => Err(ParseError::KindIsMissingAName),
        }?;

        let fields = Parser::parse_fields(&mut lex)?;

        Ok(StructureItem::Kind { name, fields })
    }

    fn parse_attr(mut lex: &mut PeekableLexer) -> Result<StructureItem, ParseError> {
        let name = match Parser::parse_name(&mut lex) {
            Ok(name) => Ok(name),
            _ => Err(ParseError::AttributeIsMissingAName),
        }?;

        let fields = Parser::parse_fields(&mut lex)?;

        Ok(StructureItem::Attribute { name, fields })
    }

    fn parse_rel(mut lex: &mut PeekableLexer) -> Result<StructureItem, ParseError> {
        let subject = match Parser::parse_name(&mut lex) {
            Ok(name) => Ok(name),
            _ => Err(ParseError::RelationExpectedSubjectToBeName),
        }?;

        let predicate = match Parser::parse_name(&mut lex) {
            Ok(name) => Ok(name),
            _ => Err(ParseError::RelationExpectedPredicateToBeName),
        }?;

        let object = match Parser::parse_name(&mut lex) {
            Ok(name) => Ok(name),
            _ => Err(ParseError::RelationExpectedObjectToBeName),
        }?;

        Ok(StructureItem::Relation {
            subject,
            predicate,
            object,
        })
    }

    fn parse_fields(mut lex: &mut PeekableLexer) -> Result<Vec<Field>, ParseError> {
        let next = lex.peek();
        if let Some(Token::OpenBrace) = next {
            lex.next();
            let mut fields = vec![];
            loop {
                match Parser::parse_field(&mut lex) {
                    Ok(field) => fields.push(field),

                    Err(ParseError::NameIsMissing) | Err(ParseError::InvalidLiteral(None)) => {
                        return Err(ParseError::IncompleteFieldBlock)
                    }

                    Err(ParseError::InvalidLiteral(Some(Token::ClosedBrace)))
                    | Err(ParseError::NameIsInvalid(Some(Token::ClosedBrace))) => break,

                    Err(e) => return Err(e),
                }
            }
            Ok(fields)
        } else {
            Ok(vec![])
        }
    }

    fn parse_field(mut lex: &mut PeekableLexer) -> Result<Field, ParseError> {
        let name = Parser::parse_name(&mut lex)?;
        let value = Parser::parse_literal(&mut lex)?;
        Ok(Field { name, value })
    }

    fn parse_literal(lex: &mut PeekableLexer) -> Result<Literal, ParseError> {
        match lex.next() {
            Some(Token::LiteralString(s)) => Ok(Literal::String(s)),
            Some(Token::Number(n)) => Ok(Literal::Number(n)),
            Some(Token::URI(uri)) => Ok(Literal::Name(Name::URI(URI::from_string(uri)))),
            Some(Token::Text(alias)) => Ok(Literal::Name(Name::Alias(alias))),
            token => Err(ParseError::InvalidLiteral(token)),
        }
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
                let mut parser = Parser::for_string("$name", $src).unwrap();
                let parsetree = parser.parse();
                let snapshot = format!(
                    r#"
input:
    {}

output:

{:#?}
"#,
                    $src, parsetree
                );
                assert_snapshot!(snapshot)
            }
        };
    }

    test!(
        parse_comment,
        " #prefix spotify:artist:2Hkut4rAAyrQxRdof7FVJq as Rush "
    );

    test!(
        parse_alias,
        " prefix spotify:artist:2Hkut4rAAyrQxRdof7FVJq as @Rush "
    );

    test!(
        parse_alias_with_something_that_isnt_a_uri,
        " prefix 2Hkut4rAAyrQxRdof7FVJq as @Rush "
    );

    test!(parse_alias_with_a_missing_uri, " prefix as @Rush ");

    test!(
        parse_alias_with_uri_but_missing_keyword_as,
        " prefix spotify:hello @Rush "
    );

    test!(
        parse_alias_with_uri_but_missing_the_aliased_name,
        " prefix spotify:hello as"
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
            # this is a prefix
            prefix spotify:kind:artist as @Artist

            kind @Artist

            attr Name

            attr spotify:field:play_count {
                @test/field 1234
            }

            kind spotify:kind:Album

            prefix spotify:kind:song as @Song

            kind Song
       "#
    );

    test!(
        parse_sample,
        r#"

prefix lore:v1 as @lore

using dota:v2022/hello/world

# hello world
attr Name

rel Name @lore/rel/asString @lore/String


        "#
    );

    test!(parse_attr_with_fields_incomplete, r#" attr Name { "#);

    test!(parse_attr_with_fields_empty, r#" attr Name {} "#);

    test!(
        parse_attr_with_fields_one,
        r#"
                attr Name {
                    @label/en "Name"
                    @label/es "Nombre"
                    @comment/en ""
                    @see-also @other/entity

                    @symmetry       :symmetric
                    @reflexivity    :reflexive
                    @lore/disjoint-with  "oops"

                    @domain      User
                    @range       @lore/string
                    @cardinality 1
                }
 "#
    );

    test!(parse_kind_with_fields_incomplete, r#" kind Name { "#);

    test!(parse_kind_with_fields_empty, r#" kind Name {} "#);

    test!(
        parse_kind_with_fields_one,
        r#" kind Name {

            fully:qualified:urn/for/name/meta/kind "world"

            @aliased/kind/string "string"
            @aliased/kind/number 1234
            @aliased/kind/uri @aliased/value
            @aliased/kind/uri f:q:uri

        } "#
    );
}
