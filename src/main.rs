use pras::lexer::Lexer;

fn main() {
    //let mut a = Lexer::new("1+2-1100 hello পলাশ");
    let mut a = Lexer::new(
        "dhori nam = ekti kaj(a)
            dekhao(a)
        sesh",
    );
    while !a.is_at_eof() {
        println!("{:?}", a.next_token());
    }
}
