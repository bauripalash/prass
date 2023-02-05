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
    "include" => Some(TokenType::Include),
    "ekti" => Some(TokenType::One),
    "then" => Some(TokenType::Then),
    "and" => Some(TokenType::And),
    "or" => Some(TokenType::Or),
    "fn" => Some(TokenType::Func),
    "let" => Some(TokenType::Let),
    "true" => Some(TokenType::True),
    "false" => Some(TokenType::False),
    "if" => Some(TokenType::If),
    "else" => Some(TokenType::Else),
    "return" => Some(TokenType::Return),
    "while" => Some(TokenType::While),
    "show" => Some(TokenType::Show),
    "end" => Some(TokenType::End),
    "break" => Some(TokenType::Break),
    _ => None,
    } 

   
}
