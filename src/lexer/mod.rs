use crate::token::{self, Token};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    read_pos: usize,
    lineno: usize,
    colno: usize,
    ch: u8,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            pos: 0,
            read_pos: 0,
            lineno: 0,
            colno: 0,
            ch: 0,
        };
        lexer.read_char();
        lexer
    }

    pub fn is_at_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn read_nunber(&mut self) -> i32 {
        let cur_pos = self.pos;
        while !self.is_at_eof() && self.ch.is_ascii_digit() {
            self.read_char()
        }
        let result = self.input[cur_pos..self.pos].parse::<i32>();
        //let result = s[cur_pos..self.pos].parse::<i32>();
        if let Ok(rs) = result {
            rs
        } else {
            0
        }

        //println!("{}" , result);
    }

    pub fn read_char(&mut self) {
        if self.read_pos >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input.as_bytes()[self.read_pos];
        }

        self.pos = self.read_pos;
        self.read_pos += 1;
        self.colno += 1;
    }

    pub fn next_token(&mut self) -> token::Token {
        let tok = match self.ch {
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'=' => Token::Eq,
            b':' => Token::Colon,
            0 => Token::Eof,
            _ => {
                if self.ch.is_ascii_digit() {
                    return Token::Number(token::NumberToken::Int(self.read_nunber().into()));
                }

                Token::Illegal
            }
        };

        self.read_char();
        tok
    }
}
