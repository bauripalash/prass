use pras::{compiler::Compiler, lexer::Lexer, parser::Parser, vm::Vm};

fn main() {
    //le&t mut a = Lexer::new("1+2-1100 hello পলাশ");
    /*let a = Lexer::new(
        "
     let a = ekti kaj(b) 1 + 2 sesh;
    ",
    );
    //while !a.is_at_eof() {
    //    println!("{:?}", a.next_token());
    //}
    let mut p = Parser::new(a);
    let _pp = p.parse_program();

    if !p.errors.is_empty() {
        for err in &p.errors {
            println!("ERR=>{}", err.msg);
        }
    } //else {
      //    println!("{pp}")
      //}
    let mut x : Instructions = Instructions::new();
    x.add_ins(code::make_ins(code::Opcode::OpAdd, &vec![]));

    x.add_ins(code::make_ins(code::Opcode::OpGetLocal, &vec![1]));
    //x.ins.push(compiler::code::make_ins(compiler::code::Opcode::OpJump, &vec![100]));
    x.add_ins(code::make_ins(code::Opcode::OpConst, &vec![2]));
    x.add_ins(make_ins(code::Opcode::OpConst, &vec![65535]));
    x.add_ins(make_ins(code::Opcode::OpClosure, &vec![65535 , 255]));

    println!("{x}")*/

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
    println!("result->{:?}", v.top_stack());
}
