use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub ttype: TokenType,
    pub literal: String,
    pub colno: usize,
    pub lineno: usize,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            ttype: TokenType::Eof,
            literal: "".to_string(),
            colno: 0,
            lineno: 0,
        }
    }
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

    pub const fn dummy() -> Self {
        Self {
            ttype: TokenType::Eof,
            literal: String::new(),
            colno: 0,
            lineno: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberToken {
    Float(f64),
    Int(i64),
}

impl Eq for NumberToken {}

impl Hash for NumberToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Int(i) => i.hash(state),
            Self::Float(f) => format!("{}", f).hash(state),
        }
    }
}

impl NumberToken {
    pub fn get_type(&self) -> bool {
        match self {
            Self::Float(..) => true,
            Self::Int(..) => false,
        }
    }
    pub fn get_as_f64(&self) -> f64 {
        match self {
            Self::Float(f) => *f,
            Self::Int(i) => *i as f64,
        }
    }

    pub fn get_as_i64(&self) -> i64 {
        match self {
            Self::Float(f) => f.round() as i64,
            Self::Int(i) => *i,
        }
    }
}

impl From<f64> for NumberToken {
    fn from(value: f64) -> Self {
        NumberToken::Float(value)
    }
}

impl From<i64> for NumberToken {
    fn from(value: i64) -> Self {
        NumberToken::Int(value)
    }
}

impl From<usize> for NumberToken {
    fn from(value: usize) -> Self {
        NumberToken::Int(value as i64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    //Illegal; Unknown symbol
    Illegal,
    Eof,       //End of file
    Plus,      // +
    Minus,     // -
    String,    // "string"
    Ident,     //let name = <>
    Number,    //100 , 1.0 , 3.14 , -123
    LSBracket, // Left Square Bracket [
    RSBracket, // Right Square Bracket ]
    Colon,     // :
    Comment,   // Comment #
    Eq,        // =
    EqEq,      // ==
    NotEq,     // !=
    Mul,       // *
    Div,       // /
    MOD,       // %
    BANG,      // !
    LT,        // <
    LTE,       // <=
    GT,        // >
    GTE,       // >=
    Semicolon, // ;
    Comma,     // ,
    Lparen,    // (
    Rparen,    // )
    Lbrace,    // {
    Rbrace,    // }

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
