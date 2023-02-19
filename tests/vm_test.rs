use std::collections::HashMap;

use pras::{
    compiler::Compiler, lexer::Lexer, obj::Object, parser::Parser, token::NumberToken, vm::Vm,
};

pub static FIB_INPUT: &str = "
    dhori fib = ekti kaj(x)
        jodi (x == 0) tahole
            ferao(0)
        nahole 
            jodi (x==1) tahole 
                ferao(1)
            nahole 
                ferao(fib(x-1) + fib(x-2))
            sesh 
        sesh 
    sesh
    fib(10)";

fn check_last_item(input: &str, output: Object) {
    let lx = Lexer::new(input);
    let mut parser = Parser::new(lx);
    let prog = parser.parse_program().expect("parser error");

    let mut com = Compiler::new();
    let bc = com.compile(prog);

    let mut vm = Vm::new(bc);
    vm.run();
    let lp = vm.last_pop();

    assert_eq!(lp.get_type(), output.get_type());

    match output {
        Object::Number { token: _, value } => match_number(&lp, &value),
        Object::String { token: _, value } => match_string(&lp, &value),

        _ => {}
    }
    //assert_eq!(lp , output)
}

fn match_number(from_vm: &Object, to_check: &NumberToken) {
    let Object::Number { token : _, value } = from_vm else{
        panic!("from_vm is not a number!");
   };

    assert_eq!(value, to_check)
}

fn match_string(from_vm: &Object, to_check: &str) {
    let Object::String { token : _, value } = from_vm else{
        panic!("from_vm is not string");
    };

    assert_eq!(value, to_check)
}

#[test]
fn test_vm_numbers() {
    let testcases = HashMap::from([
        (
            "1+2",
            Object::Number {
                token: None,
                value: pras::token::NumberToken::Int(3),
            },
        ),
        (
            "1+2*3",
            Object::Number {
                token: None,
                value: NumberToken::Int(7),
            },
        ),
    ]);

    for (k, v) in testcases {
        check_last_item(k, v)
    }
}

#[test]
fn test_vm_string() {
    let testcases = HashMap::from([
        (
            "\"1\"+\"2\"",
            Object::String {
                token: None,
                value: "12".to_string(),
            },
        ),
        (
            "\"hello\"",
            Object::String {
                token: None,
                value: String::from("hello"),
            },
        ),
    ]);

    for (k, v) in testcases {
        check_last_item(k, v)
    }
}
