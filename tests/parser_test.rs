use std::collections::HashMap;

use pras::{lexer::Lexer, parser::Parser};

fn validate_ast(input: &str, expected: &str) {
    let result: String;
    let ex: String = format!("PROG[{};]", expected);

    let lx = Lexer::new(input);
    let mut p = Parser::new(lx);
    let prog = p.parse_program();
    result = prog.expect("parsed AST has errors").to_string();
    assert_eq!(ex, result)
}

// statements are like statment<X>
// expressions are like expression(x)
// numbers are (X)

#[test]
fn test_exprs() {
    let test_cases = HashMap::from([
        ("1+2", "inf((1)+(2))"),
        ("1+2+3", "inf(inf((1)+(2))+(3))"),
        ("1+2*3", "inf((1)+inf((2)*(3)))"),
        ("show(1)", "show<(1),>"),
        ("show(1);", "show<(1),>"),
        (
            "jodi (true) tahole 1 nahole 2 sesh",
            "if(bool(true):blk<(1);>:blk<(2);>)",
        ),
        (
            "jodi (false) tahole 1 nahole 2 sesh",
            "if(bool(false):blk<(1);>:blk<(2);>)",
        ),
        (
            "jodi (1) tahole 1 nahole 2 sesh",
            "if((1):blk<(1);>:blk<(2);>)",
        ),
        ("dhori a = 1", "let<id<a|false>:(1)>"),
        ("dhori a = 1;", "let<id<a|false>:(1)>"),
        ("return(1)", "ret<(1)>"),
        ("return(1);", "ret<(1)>"),
        ("include(\"h.pank\")", "inc(str(h.pank))"),
    ]);

    for (k, v) in test_cases {
        validate_ast(k, v)
    }
}
