#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub ttype: TokenType,
    pub literal: String,
    pub colno: usize,
    pub lineno: usize,
}

impl Token {
    pub const fn new(ttype: TokenType, literal: String, colno: usize, lineno: usize) -> Self {
        Self {
            ttype,
            literal,
            colno,
            lineno,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberToken {
    Float(f64),
    Int(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
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

pub fn lookup_ident(id: &str) -> Option<TokenType> {
    match id {
        "include" | "anoyon" | "আনয়ন" => Some(TokenType::Include),
        "one" | "ekti" | "একটি" => Some(TokenType::One),
        "then" | "tahole" | "তাহলে" => Some(TokenType::Then),
        "and" | "ebong" | "এবং" => Some(TokenType::And),
        "or" | "ba" | "বা" => Some(TokenType::Or),
        "fn" | "kaj" | "কাজ" => Some(TokenType::Func),
        "let" | "dhori" | "ধরি" => Some(TokenType::Let),
        "true" | "sotti" | "সত্যি" => Some(TokenType::True),
        "false" | "mittha" | "মিথ্যা" => Some(TokenType::False),
        "if" | "jodi" | "যদি" => Some(TokenType::If),
        "else" | "nahole" | "নাহলে" => Some(TokenType::Else),
        "return" | "ferao" | "ferau" | "ফেরাও" => Some(TokenType::Return),
        "while" | "jotokhon" | "যতক্ষণ" => Some(TokenType::While),
        "show" | "dekhao" | "dekhau" | "দেখাও" => Some(TokenType::Show),
        "end" | "sesh" | "শেষ" => Some(TokenType::End),
        "break" | "bhango" | "ভাঙো" => Some(TokenType::Break),
        _ => None,
    }
}
