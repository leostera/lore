use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex("#.*(\r\n|\n)?", |lex| lex.slice()[1..].parse())]
    Comment(String),

    #[token("as")]
    As,

    #[token("prefix")]
    Prefix,

    #[token("using")]
    Using,

    #[token("kind")]
    Kind,

    #[token("attr")]
    Attribute,

    #[token("rel")]
    Relation,

    #[token("in")]
    In,

    #[token(":")]
    Colon,

    #[regex("(\"([^\"\\\\]|\\\\.)*\")", |lex| lex.slice()[1..lex.slice().len() -1].parse())]
    LiteralString(String),

    #[regex("(\"\"\"([^\"\\\\]|\\\\.)*\"\"\")", |lex| lex.slice()[3..lex.slice().len() -3].parse())]
    MultiLineLiteralString(String),

    #[token("{")]
    OpenBrace,

    #[token("}")]
    ClosedBrace,

    #[token("/")]
    Slash,

    #[regex("([a-z0-9][a-z0-9-]*:|@|:)[a-zA-Z0-9()+,-.:=@;$_!*'%/?#]+", |lex| lex.slice().parse())]
    URI(String),

    #[regex("[a-zA-Z][a-zA-Z0-9-_]*", |lex| lex.slice().parse())]
    Text(String),

    #[regex("[0-9]+", |lex| lex.slice().parse())]
    Number(u64),

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment() {
        let mut lex = Token::lexer(r#"# spotify:artist:2Hkut4rAAyrQxRdof7FVJq "#);

        assert_eq!(
            lex.next(),
            Some(Token::Comment(
                " spotify:artist:2Hkut4rAAyrQxRdof7FVJq ".to_string()
            ))
        );
    }

    #[test]
    fn literal_string_single_line() {
        let mut lex = Token::lexer(r#" "spotify:artist:2Hkut4rAAyrQxRdof7FVJq" "#);

        assert_eq!(
            lex.next(),
            Some(Token::LiteralString(
                "spotify:artist:2Hkut4rAAyrQxRdof7FVJq".to_string()
            ))
        );
    }
    #[test]
    fn literal_string_multi_line() {
        let mut lex = Token::lexer(
            r#" """
            spotify:artist:2Hkut4rAAyrQxRdof7FVJq
            more lines
            """ "#,
        );

        assert_eq!(
            lex.next(),
            Some(Token::MultiLineLiteralString("\n            spotify:artist:2Hkut4rAAyrQxRdof7FVJq\n            more lines\n            ".to_string())));
    }

    #[test]
    fn uri() {
        let mut lex = Token::lexer(r#" spotify:artist:2Hkut4rAAyrQxRdof7FVJq "#);

        assert_eq!(
            lex.next(),
            Some(Token::URI(
                "spotify:artist:2Hkut4rAAyrQxRdof7FVJq".to_string()
            ))
        );
    }

    #[test]
    fn using_namespace() {
        let mut lex = Token::lexer(r#" using spotify:schema/2021 "#);

        assert_eq!(lex.next(), Some(Token::Using));
        assert_eq!(
            lex.next(),
            Some(Token::URI("spotify:schema/2021".to_string()))
        );
    }

    #[test]
    fn prefix_uri() {
        let mut lex = Token::lexer(r#" prefix spotify:schema/2021 as @spotify "#);

        assert_eq!(lex.next(), Some(Token::Prefix));
        assert_eq!(
            lex.next(),
            Some(Token::URI("spotify:schema/2021".to_string()))
        );
        assert_eq!(lex.next(), Some(Token::As));
        assert_eq!(lex.next(), Some(Token::URI("@spotify".to_string())));
    }

    #[test]
    fn prefixed_uri() {
        let mut lex = Token::lexer(r#" @spotify/schema/2021 "#);

        assert_eq!(
            lex.next(),
            Some(Token::URI("@spotify/schema/2021".to_string()))
        );
    }

    #[test]
    fn lowercase_kind_name() {
        let mut lex = Token::lexer(r#" kind u "#);

        assert_eq!(lex.next(), Some(Token::Kind));
        assert_eq!(lex.next(), Some(Token::Text("u".to_string())));
    }

    #[test]
    fn lowercase_attr_name() {
        let mut lex = Token::lexer(r#" attr u "#);

        assert_eq!(lex.next(), Some(Token::Attribute));
        assert_eq!(lex.next(), Some(Token::Text("u".to_string())));
    }

    #[test]
    fn empty_kind() {
        let mut lex = Token::lexer(r#" kind User "#);

        assert_eq!(lex.next(), Some(Token::Kind));
        assert_eq!(lex.next(), Some(Token::Text("User".to_string())));
    }

    #[test]
    fn empty_string() {
        let mut lex = Token::lexer(r#" "" "#);
        assert_eq!(lex.next(), Some(Token::LiteralString("".to_string())));
    }

    #[test]
    fn non_empty_string() {
        let mut lex = Token::lexer(r#" "what" "#);
        assert_eq!(lex.next(), Some(Token::LiteralString("what".to_string())));
    }

    #[test]
    fn kind_with_body() {
        let mut lex = Token::lexer(r#"kind User { }"#);

        assert_eq!(lex.next(), Some(Token::Kind));
        assert_eq!(lex.next(), Some(Token::Text("User".to_string())));
        assert_eq!(lex.next(), Some(Token::OpenBrace));
        assert_eq!(lex.next(), Some(Token::ClosedBrace));
    }

    #[test]
    fn empty_attribute() {
        let mut lex = Token::lexer(r#"attr Name in User"#);

        assert_eq!(lex.next(), Some(Token::Attribute));
        assert_eq!(lex.next(), Some(Token::Text("Name".to_string())));
        assert_eq!(lex.next(), Some(Token::In));
        assert_eq!(lex.next(), Some(Token::Text("User".to_string())));
    }

    #[test]
    fn field() {
        let mut lex = Token::lexer(r#"range: integer"#);

        assert_eq!(lex.next(), Some(Token::Text("range".to_string())));
        assert_eq!(lex.next(), Some(Token::Colon));
        assert_eq!(lex.next(), Some(Token::Text("integer".to_string())));
    }

    #[test]
    fn field_block() {
        let mut lex = Token::lexer(r#" { hello world } "#);

        assert_eq!(lex.next(), Some(Token::OpenBrace));
        assert_eq!(lex.next(), Some(Token::Text("hello".to_string())));
        assert_eq!(lex.next(), Some(Token::Text("world".to_string())));
        assert_eq!(lex.next(), Some(Token::ClosedBrace));
    }

    #[test]
    fn attr_with_fields() {
        let mut lex = Token::lexer(
            r#"
                attr Name {
                    @label/en "Name"
                    @label/es "Nombre"
                    @comment/en ""
                    @see-also    @other/entity

                    @symmetry       :symmetric
                    @reflexivity    :reflexive
                    @disjoint-with  :reflexive

                    @domain      User
                    @range       @lore/string
                    @cardinality 1
                }
        "#,
        );

        assert_eq!(lex.next(), Some(Token::Attribute));
        assert_eq!(lex.next(), Some(Token::Text("Name".to_string())));
        assert_eq!(lex.next(), Some(Token::OpenBrace));
        assert_eq!(lex.next(), Some(Token::URI("@label/en".to_string())));
        assert_eq!(lex.next(), Some(Token::LiteralString("Name".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@label/es".to_string())));
        assert_eq!(lex.next(), Some(Token::LiteralString("Nombre".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@comment/en".to_string())));
        assert_eq!(lex.next(), Some(Token::LiteralString("".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@see-also".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@other/entity".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@symmetry".to_string())));
        assert_eq!(lex.next(), Some(Token::URI(":symmetric".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@reflexivity".to_string())));
        assert_eq!(lex.next(), Some(Token::URI(":reflexive".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@disjoint-with".to_string())));
        assert_eq!(lex.next(), Some(Token::URI(":reflexive".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@domain".to_string())));
        assert_eq!(lex.next(), Some(Token::Text("User".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@range".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@lore/string".to_string())));
        assert_eq!(lex.next(), Some(Token::URI("@cardinality".to_string())));
        assert_eq!(lex.next(), Some(Token::Number(1)));
        assert_eq!(lex.next(), Some(Token::ClosedBrace));
    }
}
