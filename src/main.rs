use pras::{lexer::Lexer, parser::Parser};

fn main() {
    //le&t mut a = Lexer::new("1+2-1100 hello পলাশ");
    let a = Lexer::new("1+2*3/4");
    //while !a.is_at_eof() {
    //    println!("{:?}", a.next_token());
    //}
    let mut p = Parser::new(a);
    let pp = p.parse_program();

    if !p.errors.is_empty() {
        for err in &p.errors {
            println!("ERR=>{}", err.msg);
        }
    } else {
        println!("{}", pp)
    }
}
