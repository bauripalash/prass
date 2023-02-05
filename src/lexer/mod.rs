use crate::token::{NumberToken, Token, TokenType};

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Lexer {
    input: String,
    pos: usize,
    read_pos: usize,
    lineno: usize,
    colno: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.to_string(),
            pos: 0,
            read_pos: 0,
            lineno: 1,
            colno: 0,
            ..Default::default()
        };
        lexer.read_char();
        lexer
    }

    pub fn is_at_eof(&self) -> bool {
        self.pos >= self.input.chars().count()
    }

    fn read_number(&mut self) -> i64 {
        let cur_pos = self.pos;
        while !self.is_at_eof() && self.ch.is_ascii_digit() {
            self.read_char()
        }
        let result = self.input[cur_pos..self.pos].parse::<i64>();
        //let result = s[cur_pos..self.pos].parse::<i32>();
        if let Ok(rs) = result {
            rs
        } else {
            0
        }

        //println!("{}" , result);
    }

    pub fn read_char(&mut self) {
        if self.read_pos >= self.input.chars().count() {
            self.ch = '\0';
        } else {
            self.ch = self.input.chars().nth(self.read_pos).unwrap();
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

    fn read_identifier(&mut self) -> String {
        let pos = self.pos;
        while !self.is_at_eof() && self.ch.is_ascii_alphabetic() {
            self.read_char()
        }

        let result = &self.input[pos..self.pos];
        String::from(result)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespaces();
        let result: Token;
        match self.ch {
            '+' => {
                result = Token::new(
                    TokenType::Plus,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                );
                self.read_char()
            }

            '=' => {
                result = Token::new(TokenType::Eq, self.ch.to_string(), self.colno, self.lineno);
                self.read_char();
            }
            '-' => {
                result = Token::new(
                    TokenType::Minus,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                );
                self.read_char();
            }

            ':' => {
                result = Token::new(
                    TokenType::Colon,
                    self.ch.to_string(),
                    self.colno,
                    self.lineno,
                );
                self.read_char();
            }

            '\0' => {
                result = Token::new(TokenType::Eof, self.ch.to_string(), self.colno, self.lineno);
                self.read_char();
            }

            _ => {
                if self.ch.is_ascii_digit() {
                    let colno = self.colno;
                    let lineno = self.lineno;
                    let n = self.read_number();

                    result = Token {
                        ttype: TokenType::Number(NumberToken::Int(n)),
                        literal: n.to_string(),
                        colno,
                        lineno,
                    }
                } else if self.ch.is_ascii_alphabetic() {
                    let colno = self.colno;
                    let lineno = self.lineno;
                    let id = self.read_identifier();

                    result = Token::new(TokenType::Ident(id.clone()), id, colno, lineno)
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
        //self.read_char();
        result
    }
}
