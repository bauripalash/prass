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

const KW_NAMES: [&str; 16] = [
    "include", "a", "then", "and", "or", "fn", "let", "true", "false", "if", "else", "return",
    "while", "show", "end", "break",
];
const KW_TOKS: [TokenType; 16] = [
    TokenType::Include,
    TokenType::One,
    TokenType::Then,
    TokenType::And,
    TokenType::Or,
    TokenType::Func,
    TokenType::Let,
    TokenType::True,
    TokenType::False,
    TokenType::If,
    TokenType::Else,
    TokenType::Return,
    TokenType::While,
    TokenType::Show,
    TokenType::End,
    TokenType::Break,
];

pub fn lookup_ident(id: &str) -> Option<TokenType> {
    let name_index = KW_NAMES.iter().position(|&a| a == id);
    name_index.map(|pos| KW_TOKS[pos].clone())
}
