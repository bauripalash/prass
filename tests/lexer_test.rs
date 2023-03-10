use pras::lexer::Lexer;
use pras::token::TokenType;

#[test]
fn test_lexer_next_token() {
    let input = "a+b;
        let name = \"name\"
        ";

    let ex_tok_types = vec![
        TokenType::Ident,
        TokenType::Plus,
        TokenType::Ident,
        TokenType::Semicolon,
        TokenType::Let,
        TokenType::Ident,
        TokenType::Eq,
        TokenType::String,
    ];

    let mut lx = Lexer::new(input);

    for ett in ex_tok_types {
        if let Ok(t) = lx.next_token() {
            assert_eq!(ett, t.ttype)
        }
    }
}
