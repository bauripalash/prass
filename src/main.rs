use pras::lexer::Lexer;

fn main() {
    let mut a = Lexer::new("1+2-1100");
    while !a.is_at_eof() {
        println!("{:?}", a.next_token());
    }
}
