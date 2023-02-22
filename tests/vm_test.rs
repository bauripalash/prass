use std::collections::HashMap;

use pras::{
    compiler::Compiler,
    lexer::Lexer,
    obj::{Object, BOOL_OBJ, NUMBER_OBJ, STRING_OBJ},
    parser::Parser,
    vm::Vm,
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

pub static FIB_INPUT_ITER: &str = "
    dhori fib = ekti kaj(x)
        dhori i = 0
        dhori x = 0
        dhori y = 1
        dhori z = 0
        jotkhon i < x 
            dhori z = x + y
            dhori x = y 
            dhori z = z
            dhori i = i + 1 
        sesh 
    sesh
    
    fib(10)
";

fn get_obj(input: &str) -> Object {
    let lx = Lexer::new(input);
    let mut parser = Parser::new(lx);
    let prog = parser.parse_program().expect("parser error");

    let mut com = Compiler::new();
    let bc = com.compile(prog);

    let mut vm = Vm::new(bc);
    vm.run();
    vm.last_pop()
}

fn check_last_item_bool(input: &str, output: bool) {
    let obj = get_obj(input);
    assert_eq!(obj.get_type(), BOOL_OBJ);
    let Object::Bool { token : _, value } = obj else{
        panic!("check_last_item_bool -> obj not bool")
    };

    assert_eq!(value, output)
}

fn check_last_item_int(input: &str, output: i64) {
    let obj = get_obj(input);
    assert_eq!(obj.get_type(), NUMBER_OBJ);
    let Object::Number { token : _, value } = obj else{
        panic!("check_last_item_int  -> obj not int")
    };

    assert_eq!(value.is_int(), true);

    assert_eq!(value.get_as_i64(), output)
}

fn check_last_item_float(input: &str, output: f64) {
    let obj = get_obj(input);
    assert_eq!(obj.get_type(), NUMBER_OBJ);
    let Object::Number { token : _, value } = obj else{
        panic!("check_last_item_float -> obj not float")
    };

    assert_eq!(value.is_int(), false);

    assert_eq!(value.get_as_f64(), output)
}

fn check_last_item_string(input: &str, output: &str) {
    let obj = get_obj(input);
    assert_eq!(obj.get_type(), STRING_OBJ);
    let Object::String { token : _, value } = obj else{
        panic!("check_last_item_float -> obj not float")
    };

    assert_eq!(value.as_str(), output)
}

#[test]
fn test_vm_numbers_int() {
    let testcases = HashMap::from([
        ("1", 1),
        ("10", 10),
        ("1-2", -1),
        ("1+2", 3),
        ("1+2*3", 7),
        ("4/2", 2),
        ("5/2", 2),
        ("50/2 * 2 + 10 - 5", 55),
        (FIB_INPUT, 55),
        (
            "dhori a = ekti kaj() dhori a = 1; ferao(a) sesh;
         a()",
            1,
        ),
        (
            "dhori addr = ekti kaj(a,b)
            ekti kaj(c)
                ferao(a + b + c)
            sesh
          sesh
          let newaddr = addr(1,2)
          addr(8)
          ",
            11,
        ),
    ]);

    for (k, v) in testcases {
        check_last_item_int(k, v)
    }
}

#[test]
fn test_vm_numbers_float() {
    let testcases = HashMap::from([
        ("5.0/2", 2.5),
        ("100.0/2", 50.0),
        ("22.0/7", 3.142857142857143),
        ("11.0/3+4", 7.666666666666666),
    ]);

    for (k, v) in testcases {
        check_last_item_float(k, v)
    }
}

#[test]
fn test_vm_string() {
    let testcases = HashMap::from([
        ("\"1\"+\"2\"", "12"),
        ("\"hello\"", "hello"),
        ("\"100\"", "100"),
        ("jodi (true) tahole \"true\" nahole sesh", "true"),
        ("jodi (1+2 > 3) tahole \"true\" nahole \"false\"", "false"),
    ]);

    for (k, v) in testcases {
        check_last_item_string(k, v)
    }
}

#[test]
fn test_vm_bool() {
    let testcases = HashMap::from([("true", true)]);

    for (k, v) in testcases {
        check_last_item_bool(k, v)
    }
}
