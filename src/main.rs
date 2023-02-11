use pras::{compiler::Compiler, lexer::Lexer, parser::Parser, vm::Vm};

fn main() {
    let a = Lexer::new("100+200");
    let mut parser = Parser::new(a);
    let parsed_program = parser.parse_program();

    if parser.errors.is_empty() {
        println!("parse_done!");
    } else {
        for err in &parser.errors {
            println!("Err => {}", err.msg);
        }
    }

    let mut cm = Compiler::new();
    let c = cm.compile(parsed_program);
    //println!("{c:?}");
    let mut v = Vm::new(c);
    v.run();
    println!("{}", v.instructions.to_string());
    println!("result->{:?}", v.last_pop());
}
