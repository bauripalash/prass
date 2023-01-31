#[derive(Debug, Clone, PartialEq)]
pub enum NumberToken {
    Float(f64),
    Int(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    //Illegal; Unknown symbol
    Illegal,
    Eof,                 //End of file
    Plus,                // +
    Minus,               // -
    String,              // "string"
    Ident(String),       //let name = <>
    Number(NumberToken), //100 , 1.0 , 3.14 , -123
    LSBracket,           // Left Square Bracket [
    RSBracket,           // Right Square Bracket ]
    Colon,               // :
    Comment,             // Comment #
    Eq,                  // =
    EqEq,                // ==
    NotEq,               // !=
    Mul,                 // *
    Div,                 // /
    MOD,                 // %
    BANG,                // !
    LT,                  // <
    LTE,                 // <=
    GT,                  // >
    GTE,                 // >=
    Semicolon,           // ;
    Comma,               // ,
    Lparen,              // (
    Rparen,              // )
    Lbrace,              // {
    Rbrace,              // }

    //Keywords
    Include, // Include Keyword
    One,     // ekti
    Then,    // tahole
    And,     //and / ebong
    Or,      // or / ba
    Func,    // function
    Let,
    True,
    False,
    If,
    Else,
    Return,
    While,
    Show,
    End,
    Break,
}

const KW_NAMES: [&str; 16] = [
    "include", "a", "then", "and", "or", "fn", "let", "true", "false", "if", "else", "return",
    "while", "show", "end", "break",
];
const KW_TOKS: [Token; 16] = [
    Token::Include,
    Token::One,
    Token::Then,
    Token::And,
    Token::Or,
    Token::Func,
    Token::Let,
    Token::True,
    Token::False,
    Token::If,
    Token::Else,
    Token::Return,
    Token::While,
    Token::Show,
    Token::End,
    Token::Break,
];

pub fn lookup_ident(id: &str) -> Option<Token> {
    let name_index = KW_NAMES.iter().position(|&a| a == id);
    name_index.map(|pos| KW_TOKS[pos].clone())
}
