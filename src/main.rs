use pras::{lexer::Lexer, parser::Parser};

fn main() {
    //let mut a = Lexer::new("1+2-1100 hello পলাশ");
    let a = Lexer::new("let a = 1+2.1");
    //while !a.is_at_eof() {
    //    println!("{:?}", a.next_token());
    //}
    let mut p = Parser::new(a);
    println!("{:#?}", p.parse_program())
}
