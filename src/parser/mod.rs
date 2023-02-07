use std::rc::Rc;

use crate::{
    ast::{self, Program, Stmt},
    lexer::Lexer,
    token::{self, Token, TokenType},
};

#[allow(dead_code)]
const P_LOWEST: usize = 1;
const P_EQUALS: usize = 2;
const P_LOGIC: usize = 3;
const P_LTGT: usize = 4;
const P_SUM: usize = 5;
const P_PROD: usize = 6;
const P_PREFIX: usize = 7;
const P_CALL: usize = 8;
const P_INDEX: usize = 9;

pub const fn get_precedences(tt: &TokenType) -> usize {
    match tt {
        TokenType::EqEq | TokenType::NotEq => P_EQUALS,
        TokenType::And | TokenType::Or => P_LOGIC,
        TokenType::LT | TokenType::LTE | TokenType::GT | TokenType::GTE => P_LTGT,
        TokenType::Plus | TokenType::Minus => P_SUM,
        TokenType::Div | TokenType::Mul | TokenType::MOD => P_PROD,
        TokenType::Lparen => P_CALL,
        TokenType::LSBracket => P_INDEX,

        _ => P_LOWEST,
    }
}

#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

#[derive(Debug)]
pub struct Parser<'lx> {
    lexer: Lexer<'lx>,
    curtok: Token,
    peektok: Token,
    errors: Vec<Error>,
}

impl<'lx> Parser<'lx> {
    pub fn new(lexer: Lexer<'lx>) -> Self {
        let mut p = Self {
            lexer,
            curtok: Token::dummy(),
            peektok: Token::dummy(),
            errors: vec![],
        };

        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) -> Token {
        self.curtok = self.peektok.clone();
        self.peektok = self.lexer.next_token();
        self.curtok.clone()
    }

    fn is_curtok(&self, tok: &TokenType) -> bool {
        return self.curtok.ttype == *tok;
    }

    fn is_peektok(&self, tok: &TokenType) -> bool {
        return self.peektok.ttype == *tok;
    }

    fn peek_prec(&self) -> usize {
        get_precedences(&self.peektok.ttype)
    }

    fn cur_prec(&self) -> usize {
        get_precedences(&self.curtok.ttype)
    }

    fn peek(&mut self, tok: &TokenType) -> bool {
        if self.is_peektok(tok) {
            self.next_token();
            true
        } else {
            self.peek_error(&tok);
            false
        }
    }

    fn peek_error(&mut self, tok: &TokenType) {
        self.errors.push(Error {
            msg: format!("Expected {:?} but got {:?}", tok, self.curtok.ttype),
        });
    }

    pub fn parse_program(&mut self) -> ast::Program {
        let stms = self.parse_stmts();
        Program { stmts: stms }
    }

    fn parse_stmts(&mut self) -> Vec<Rc<ast::Stmt>> {
        let mut stmts: Vec<Rc<ast::Stmt>> = Vec::new();
        match self.curtok.ttype {
            TokenType::Let => stmts.push(self.parse_let_stmt()),
            _ => stmts.push(self.parse_expr_stmt()),
        }
        stmts
    }

    fn parse_let_stmt(&mut self) -> Rc<Stmt> {
        let ctok = self.curtok.clone();
        self.next_token();

        let id = ast::Identifier {
            token: self.curtok.clone(),
            name: self.curtok.literal.clone(),
        };

        if self.peek(&TokenType::Eq) {
            self.next_token();
        }

        let val = self.parse_expr(P_LOWEST);

        if self.is_peektok(&TokenType::Semicolon) {
            self.next_token();
        }

        Rc::new(ast::Stmt::LetStmt {
            token: ctok.clone(),
            name: id,
            value: val,
        })
    }
    fn parse_expr_stmt(&mut self) -> Rc<Stmt> {
        let ex = Rc::new(ast::Stmt::ExprStmt {
            token: self.curtok.clone(),
            expr: self.parse_expr(P_LOWEST),
        });

        if self.is_peektok(&TokenType::Semicolon) {
            self.next_token();
        }

        return ex;
    }

    fn parse_expr(&mut self, prec: usize) -> Rc<ast::Expr> {
        let mut left_expr = self.parse_prefix_expr();

        while !self.is_peektok(&TokenType::Semicolon) && prec < self.peek_prec() {
            self.next_token();
            //println!("{:?}->{:?}" ,self.curtok , "");
            let infx = self.parse_infix_expr(left_expr.clone());
            if let Ok(ix) = infx {
                left_expr = ix;
            } else {
                return left_expr;
            }
        }

        left_expr
    }

    fn got_error_jump(&mut self, msg: String) {
        self.errors.push(Error { msg });
        self.next_token();
    }

    fn parse_prefix_expr(&mut self) -> Rc<ast::Expr> {
        match self.curtok.ttype {
            TokenType::Ident => self.parse_identifier(),
            TokenType::Number => self.parse_number(),
            TokenType::String => self.parse_string_lit(),
            TokenType::True | TokenType::False => self.parse_bool(),
            _ => {
                self.got_error_jump(format!(
                    "Unknown Prefix; Unexpected token {:?}",
                    self.curtok.ttype
                ));
                Rc::new(ast::Expr::ErrExpr)
            }
        }
    }

    fn parse_infix_expr(&mut self, left: Rc<ast::Expr>) -> Result<Rc<ast::Expr>, bool> {
        match self.curtok.ttype {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Mul
            | TokenType::Div
            | TokenType::EqEq
            | TokenType::NotEq
            | TokenType::LT
            | TokenType::GT
            | TokenType::MOD => Ok(Rc::new(self.parse_infix_op(left))),
            _ => Err(true),
        }
    }

    fn parse_infix_op(&mut self, left: Rc<ast::Expr>) -> ast::Expr {
        //println!("PREFIX_OP -> {:?}->{:?}" , left , self.curtok);
        let op = self.curtok.clone();
        let prec = get_precedences(&op.ttype);
        self.next_token();
        let right = self.parse_expr(prec);

        ast::Expr::InfixExpr {
            token: op.clone(),
            left,
            op: op.clone(),
            right,
        }
    }

    fn parse_identifier(&mut self) -> Rc<ast::Expr> {
        Rc::new(ast::Expr::IdentExpr {
            token: self.curtok.clone(),
            value: self.curtok.literal.clone(),
        })
    }

    fn parse_string_lit(&mut self) -> Rc<ast::Expr> {
        Rc::new(ast::Expr::StringExpr {
            token: self.curtok.clone(),
            value: self.curtok.literal.clone(),
        })
    }

    fn parse_bool(&mut self) -> Rc<ast::Expr> {
        Rc::new(ast::Expr::BoolExpr {
            token: self.curtok.clone(),
            value: self.is_curtok(&TokenType::True),
        })
    }

    fn parse_break(&mut self) -> Rc<ast::Expr> {
        Rc::new(ast::Expr::Break {
            token: self.curtok.clone(),
            value: self.curtok.literal.clone(),
        })
    }

    fn parse_number(&mut self) -> Rc<ast::Expr> {
        let curtok = self.curtok.clone();
        let curtok_lit = self.curtok.literal.clone();
        let nl: Vec<&str> = curtok_lit.split('.').collect();

        if nl.len() == 1 {
            let v = curtok_lit.parse::<i64>().unwrap();
            return Rc::new(ast::Expr::NumExpr {
                token: curtok,
                value: token::NumberToken::Int(v),
                is_int: true,
            });
        } else {
            // Should be == 2 : TO DO -> Check
            let v = curtok_lit.parse::<f64>().unwrap();
            return Rc::new(ast::Expr::NumExpr {
                token: curtok,
                value: token::NumberToken::Float(v),
                is_int: false,
            });
        }
    }
}
