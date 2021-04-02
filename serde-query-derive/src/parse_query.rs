use logos::Logos;

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    #[token(r#"."#)]
    Dot,
    #[token(r#"["#)]
    OpenBracket,
    #[token(r#"]"#)]
    CloseBracket,
    #[regex(r#"[a-zA-Z_][0-9a-zA-Z_]*"#)]
    // https://github.com/maciejhirsz/logos/issues/133#issuecomment-619444615
    #[regex(r#""(?:[^"]|\\")*""#)]
    Field,
    #[regex(r#"[0-9]+"#)]
    Index,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lex() {
        use Token::*;

        let lexer = Token::lexer(r#". [ "kubernetes_clusters" ].id.[0 ]"#);
        let tokens: Vec<Token> = lexer.collect();
        assert_eq!(
            tokens,
            [
                Dot,
                OpenBracket,
                Field,
                CloseBracket,
                Dot,
                Field,
                Dot,
                OpenBracket,
                Index,
                CloseBracket
            ]
        );
    }
}
