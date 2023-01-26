use crate::token::{self, Token};

#[allow(dead_code)]
pub struct Lexer<'a> {
    input : &'a str,
    pos : usize,
    read_pos : usize,
    lineno : usize,
    colno : usize,
    ch : u8,
}

impl<'a> Lexer<'a>{
    pub fn new(input : &'a str) -> Self{
        let mut lexer = Lexer{
            input,
            pos: 0,
            read_pos:0,
            lineno:0,
            colno:0,
            ch: 0,
        };
        lexer.read_char();
        lexer
    }

    pub fn is_at_eof(&self) -> bool {
        
        self.pos >= self.input.len()

    }

    pub fn read_char(&mut self){
        if self.read_pos >= self.input.len(){
            self.ch = 0;
        }else{
            self.ch = self.input.as_bytes()[self.read_pos];
        }

        self.pos = self.read_pos;
        self.read_pos+=1;
        self.colno+=1;
    }

    pub fn next_token(&mut self) -> token::Token {
        let tok = match self.ch {
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'=' => Token::Eq,
            0 => Token::Eof,
            _ => Token::Illegal
        };

        self.read_char();
        tok
    }
}
