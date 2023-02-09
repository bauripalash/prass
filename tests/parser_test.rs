use std::collections::HashMap;

use pras::{lexer::Lexer, parser::Parser};

fn validate_ast(input: &str, expected: &str) {
    let result: String;
    let ex: String = format!("PROG[{};]", expected);

    let lx = Lexer::new(input);
    let mut p = Parser::new(lx);
    let prog = p.parse_program();
    result = prog.to_string();
    assert_eq!(ex, result)
}

#[test]
fn test_exprs() {
    let test_cases = HashMap::from([
        ("1+2", "inf((1)+(2))"),
        ("1+2+3", "inf(inf((1)+(2))+(3))"),
        ("1+2*3", "inf((1)+inf((2)*(3)))"),
    ]);

    for (k, v) in test_cases {
        validate_ast(k, v)
    }
}
