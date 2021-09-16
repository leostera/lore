use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex("@[a-zA-Z]+", |lex| lex.slice()[1..].parse())]
    Annotation(String),

    #[token("as")]
    As,

    #[token("use")]
    Use,

    #[token("kind")]
    Kind,

    #[token("attr")]
    Attribute,

    #[token("in")]
    In,

    #[token(":")]
    Colon,

    #[regex("\"([^\"\\\\]|\\\\.)*\"", |lex| lex.slice()[1..lex.slice().len() -1].parse())]
    LiteralString(String),

    #[token("{")]
    OpenBrace,

    #[token("}")]
    ClosedBrace,

    #[regex("[a-z0-9][a-z0-9-]*:[a-zA-Z0-9()+,-.:=@;$_!*'%/?#]+", |lex| lex.slice().parse())]
    URI(String),

    #[regex("[a-zA-Z]+", |lex| lex.slice().parse())]
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
    fn use_uri() {
        let mut lex = Token::lexer(r#" use spotify:schema/2021/artist as Artist "#);

        assert_eq!(lex.next(), Some(Token::Use));
        assert_eq!(
            lex.next(),
            Some(Token::URI("spotify:schema/2021/artist".to_string()))
        );
        assert_eq!(lex.next(), Some(Token::As));
        assert_eq!(lex.next(), Some(Token::Text("Artist".to_string())));
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
    fn standalone_annotation() {
        let mut lex = Token::lexer(r#" @attr "#);

        assert_eq!(lex.next(), Some(Token::Annotation("attr".to_string())));
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
    fn kind_with_annotations() {
        let mut lex = Token::lexer(
            r#"kind User {
            @en "documentation"
            @es "documentacion"
        }"#,
        );

        assert_eq!(lex.next(), Some(Token::Kind));
        assert_eq!(lex.next(), Some(Token::Text("User".to_string())));
        assert_eq!(lex.next(), Some(Token::OpenBrace));
        assert_eq!(lex.next(), Some(Token::Annotation("en".to_string())));
        assert_eq!(
            lex.next(),
            Some(Token::LiteralString("documentation".to_string()))
        );
        assert_eq!(lex.next(), Some(Token::Annotation("es".to_string())));
        assert_eq!(
            lex.next(),
            Some(Token::LiteralString("documentacion".to_string()))
        );
        assert_eq!(lex.next(), Some(Token::ClosedBrace));
    }

    #[test]
    fn attr_with_fields() {
        let mut lex = Token::lexer(
            r#"attr Name in User {
            range: string
            cardinality: 1
        }"#,
        );

        assert_eq!(lex.next(), Some(Token::Attribute));
        assert_eq!(lex.next(), Some(Token::Text("Name".to_string())));
        assert_eq!(lex.next(), Some(Token::In));
        assert_eq!(lex.next(), Some(Token::Text("User".to_string())));
        assert_eq!(lex.next(), Some(Token::OpenBrace));
        assert_eq!(lex.next(), Some(Token::Text("range".to_string())));
        assert_eq!(lex.next(), Some(Token::Colon));
        assert_eq!(lex.next(), Some(Token::Text("string".to_string())));
        assert_eq!(lex.next(), Some(Token::Text("cardinality".to_string())));
        assert_eq!(lex.next(), Some(Token::Colon));
        assert_eq!(lex.next(), Some(Token::Number(1)));
        assert_eq!(lex.next(), Some(Token::ClosedBrace));
    }
}
