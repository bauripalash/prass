use crate::{
    bn::{is_bn_char, is_bn_num, parse_bn_num},
    errorhelper::ErrorHelper,
    token::{lookup_ident, Token, TokenType},
};
use std::rc::Rc;

fn charlist_to_string(charlist: &[char]) -> String {
    String::from_iter(charlist.iter())
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Lexer<'a> {
    input: &'a str,
    charlist: Vec<char>,
    pos: usize,
    read_pos: usize,
    lineno: usize,
    colno: usize,
    ch: char,
    pub eh: ErrorHelper,
}

pub struct LexerError {
    pub token: Option<Token>,
    pub msg: String,
}

impl<'a> Lexer<'a> {
    pub fn new(inp: &'a str) -> Self {
        let mut lexer = Lexer {
            input: inp,
            charlist: inp.chars().collect(),
            pos: 0,
            read_pos: 0,
            lineno: 1,
            colno: 0,
            ..Default::default()
        };
        lexer.eh = ErrorHelper::new(inp);
        lexer.read_char();
        lexer
    }

    pub fn is_at_eof(&self) -> bool {
        self.pos >= self.charlist.len()
    }

    fn peek(&self) -> char {
        if self.read_pos >= self.charlist.len() {
            '\0'
        } else {
            self.charlist[self.read_pos]
        }
    }

    fn read_number(&mut self) -> Option<Token> {
        //println!("{:?}" , self.input.chars());
        let cur_pos = self.pos;
        let colno = self.colno;
        let lineno = self.lineno;
        while self.ch.is_ascii_digit() || is_bn_num(self.ch) {
            self.read_char();
        }

        if self.ch == '.' {
            self.read_char();
            while self.ch.is_ascii_digit() || is_bn_num(self.ch) {
                self.read_char();
            }
        }

        let raw_number: String =
            parse_bn_num(&charlist_to_string(&self.charlist[cur_pos..self.pos]));

        if raw_number.is_ascii() {
            Some(Token::new(TokenType::Number, raw_number, colno, lineno))
        } else {
            None
        }
    }
    fn read_identifier(&mut self) -> String {
        let pos = self.pos;
        while !self.is_at_eof() && (self.ch.is_ascii_alphabetic() || is_bn_char(self.ch)) {
            self.read_char()
        }

        if self.ch == '.' {
            self.read_char();
            while !self.is_at_eof() && (self.ch.is_ascii_alphabetic() || is_bn_char(self.ch)) {
                self.read_char()
            }
        }

        charlist_to_string(&self.charlist[pos..self.pos]).to_string()
    }

    fn read_string(&mut self) -> Token {
        let pos = self.pos + 1;
        let colno = self.colno;
        let lineno = self.lineno;

        loop {
            self.read_char();
            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }

        let slit = &self.charlist[pos..self.pos];

        Token {
            ttype: TokenType::String,
            literal: charlist_to_string(slit).to_string(),
            colno,
            lineno,
        }
    }

    pub fn read_char(&mut self) {
        if self.read_pos >= self.charlist.len() {
            self.ch = '\0';
        } else {
            self.ch = self.charlist[self.read_pos];
        }

        self.pos = self.read_pos;
        self.read_pos += 1;
        self.colno += 1;
    }

    fn skip_whitespaces(&mut self) {
        while self.ch.is_whitespace() {
            if self.ch == '\n' {
                self.lineno += 1;
            }
            self.read_char();
        }
    }

    fn skip_comment(&mut self) {
        loop {
            if self.peek() == '\n' || self.peek() == '\0' {
                break;
            }

            self.read_char();
        }
        self.read_char();
        self.skip_whitespaces();
    }

    pub fn next_token(&mut self) -> Result<Rc<Token>, LexerError> {
        self.skip_whitespaces();
        if self.ch == '#' {
            self.skip_comment();
        }
        let result: Token;
        match self.ch {
            '+' => {
                result = Token::new(
                    TokenType::Plus,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                );
                //self.read_char()
            }
            '-' => {
                result = Token::new(
                    TokenType::Minus,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                );
                //self.read_char();
            }

            '*' => {
                result = Token::new(TokenType::Mul, self.ch.to_string(), self.colno, self.lineno);
                //self.read_char();
            }

            '/' => {
                result = Token::new(TokenType::Div, self.ch.to_string(), self.colno, self.lineno)
            }

            '=' => {
                if self.peek() == '=' {
                    let ch = self.ch.to_string();
                    self.read_char();
                    result = Token::new(
                        TokenType::EqEq,
                        ch + &self.ch.to_string(),
                        self.colno,
                        self.lineno,
                    )
                } else {
                    result =
                        Token::new(TokenType::Eq, self.ch.to_string(), self.colno, self.lineno);
                }
                //self.read_char();
            }

            ';' => {
                result = Token::new(
                    TokenType::Semicolon,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            ',' => {
                result = Token::new(
                    TokenType::Comma,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            '<' => {
                if self.peek() == '=' {
                    let ch = self.ch.to_string();
                    self.read_char();
                    result = Token::new(
                        TokenType::LTE,
                        ch + &self.ch.to_string(),
                        self.colno,
                        self.lineno,
                    )
                } else {
                    result = Token::new(TokenType::LT, self.ch.to_string(), self.colno, self.lineno)
                }
            }
            '>' => {
                if self.peek() == '=' {
                    let ch = self.ch.to_string();
                    self.read_char();
                    result = Token::new(
                        TokenType::GTE,
                        ch + &self.ch.to_string(),
                        self.colno,
                        self.lineno,
                    )
                } else {
                    result = Token::new(TokenType::GT, self.ch.to_string(), self.colno, self.lineno)
                }
            }
            '(' => {
                result = Token::new(
                    TokenType::Lparen,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            ')' => {
                result = Token::new(
                    TokenType::Rparen,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            '{' => {
                result = Token::new(
                    TokenType::Lbrace,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            '}' => {
                result = Token::new(
                    TokenType::Rbrace,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            '[' => {
                result = Token::new(
                    TokenType::LSBracket,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            ']' => {
                result = Token::new(
                    TokenType::RSBracket,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                )
            }
            '%' => {
                result = Token::new(TokenType::MOD, self.ch.to_string(), self.colno, self.lineno)
            }
            '!' => {
                if self.peek() == '=' {
                    let ch = self.ch.to_string();
                    self.read_char();

                    result = Token::new(
                        TokenType::NotEq,
                        ch + &self.ch.to_string(),
                        self.colno,
                        self.lineno,
                    )
                } else {
                    result = Token::new(
                        TokenType::BANG,
                        self.ch.to_string(),
                        self.colno,
                        self.lineno,
                    )
                }
            }
            '"' => result = self.read_string(),

            ':' => {
                result = Token::new(
                    TokenType::Colon,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                );
                //self.read_char();
            }

            '\0' => {
                result = Token::new(TokenType::Eof, self.ch.to_string(), self.colno, self.lineno);
                //self.read_char();
            }

            _ => {
                if self.ch.is_ascii_digit() || is_bn_num(self.ch) {
                    let raw_number = self.read_number();
                    if let Some(n) = raw_number {
                        return Ok(Rc::new(n));
                    } else {
                        return Err(LexerError {
                            token: None,
                            msg: format!("Invalid number literal -> {raw_number:?}"),
                        });
                    }
                } else if (self.ch.is_ascii_alphabetic() || is_bn_char(self.ch))
                    && !is_bn_num(self.ch)
                {
                    let colno = self.colno;
                    let lineno = self.lineno;
                    let id = self.read_identifier();

                    if let Some(kw) = lookup_ident(id.as_str()) {
                        return Ok(Rc::new(Token::new(kw, id, colno, lineno)));
                    }

                    return Ok(Rc::new(Token::new(TokenType::Ident, id, colno, lineno)));
                } else {
                    result = Token::new(
                        TokenType::Illegal,
                        self.ch.to_string(),
                        self.colno,
                        self.lineno,
                    );
                    self.read_char();
                }
            }
        };
        self.read_char();
        Ok(Rc::new(result))
    }
}
