use std::collections::HashMap;

use pras::{compiler::Compiler, lexer::Lexer, parser::Parser};

fn check_compiler_instr(src: &str, output: &str) {
    let lx = Lexer::new(src);
    let mut p = Parser::new(lx);
    let prog = p.parse_program();

    let mut cm = Compiler::new();
    let bc = cm.compile(prog.expect("parsed AST has errors"));
    assert_eq!(bc.instructions.to_string(), output.to_string())
}

#[test]
fn test_compiler() {
    let testcases = HashMap::from([
        (
            "1+2",
            "0000 OpConst 0\n0003 OpConst 1\n0006 OpAdd\n0007 OpPop\n",
        ),
        (
            "dhori global = 55

 ekti kaj()
    dhori a = 66

    ekti kaj()
        dhori b = 77
        
        ekti kaj()
            dhori c = 88 
            global + a + b + c 
        sesh
    sesh
sesh",
            "0000 OpConst 0\n0003 OpSetGlobal 0\n0006 OpClosure 6 0\n0010 OpPop\n",
        ),
    ]);

    for (k, v) in testcases {
        check_compiler_instr(k, v)
    }
}
