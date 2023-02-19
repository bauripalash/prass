use std::collections::HashMap;

use pras::token::{lookup_ident, NumberToken, TokenType};

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

#[test]
fn test_number_token() {
    assert_eq!(
        NumberToken::Int(5) + NumberToken::Int(4),
        NumberToken::Int(9)
    );
    assert_eq!(
        NumberToken::Int(9) - NumberToken::Int(4),
        NumberToken::Int(5)
    );
    assert_eq!(
        NumberToken::Int(9) - NumberToken::Int(-1),
        NumberToken::Int(10)
    );
    assert_eq!(NumberToken::from(9), NumberToken::Int(9));
    assert_eq!(
        NumberToken::from(8) * NumberToken::from(-8),
        NumberToken::from(-64)
    );
    assert_eq!(
        NumberToken::from(5) / NumberToken::from(2),
        NumberToken::from(2)
    );
    assert_eq!(
        NumberToken::from(5.0) / NumberToken::from(2),
        NumberToken::from(2.5)
    );
}
