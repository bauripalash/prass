use pras::lexer::Lexer;
use pras::token::TokenType;

#[test]
fn test_lexer_next_token() {
    let input = "a+b;
        let name = \"name\"
        ";

    let ex_tok_types = vec![
        TokenType::Ident("a".to_string()),
        TokenType::Plus,
        TokenType::Ident("b".to_string()),
        TokenType::Semicolon,
        TokenType::Let,
        TokenType::Ident("name".to_string()),
        TokenType::Eq,
        TokenType::String,
    ];

    let mut lx = Lexer::new(input);

    for ett in ex_tok_types {
        let toktype = lx.next_token().ttype;
        assert_eq!(ett, toktype)
    }
}
