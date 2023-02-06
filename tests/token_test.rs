use std::collections::HashMap;

use pras::token::{lookup_ident, TokenType};

#[test]
fn test_lookup_ident() {
    let test_cases = HashMap::from([
        ("include", TokenType::Include),
        ("ekti", TokenType::One),
        ("then", TokenType::Then),
        ("and", TokenType::And),
        ("or", TokenType::Or),
        ("fn", TokenType::Func),
        ("let", TokenType::Let),
        ("true", TokenType::True),
        ("false", TokenType::False),
        ("if", TokenType::If),
        ("else", TokenType::Else),
        ("return", TokenType::Return),
        ("while", TokenType::While),
        ("show", TokenType::Show),
        ("end", TokenType::End),
        ("break", TokenType::Break),
    ]);

    for (k, v) in test_cases {
        assert_eq!(lookup_ident(k).unwrap(), v);
    }
}
