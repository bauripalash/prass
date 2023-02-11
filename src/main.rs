use pras::{compiler::Compiler, lexer::Lexer, parser::Parser, vm::Vm};

fn main() {
    let a = Lexer::new("jodi (true) tahole 1 nahole 2 sesh; 3333");
    let mut parser = Parser::new(a);
    let parsed_program = parser.parse_program();

    if parser.errors.is_empty() {
        println!("{parsed_program}");
    } else {
        for err in &parser.errors {
            println!("Err => {}", err.msg);
        }
    }

    let mut cm = Compiler::new();
    let c = cm.compile(parsed_program);
    println!("{}", c.instructions);
    let mut v = Vm::new(c);
    v.run();
    //println!("{}", v.instructions);
    //println!("result->{:?}", v.last_pop());
}
