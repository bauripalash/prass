use std::rc::Rc;

use crate::{
    ast::{self, Identifier, Program, Stmt},
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
    pub errors: Vec<Error>,
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

    //
    //
    //  Helper Functions
    //
    //

    fn next_token(&mut self) -> Token {
        if self.peektok.ttype == TokenType::Illegal {
            self.errors.push(Error {
                msg: format!("Illegal token ->{:?}", self.peektok),
            })
        }
        self.curtok = self.peektok.clone();
        self.peektok = self.lexer.next_token();
        self.curtok.clone()
    }

    fn is_curtok(&self, tok: &TokenType) -> bool {
        self.curtok.ttype == *tok
    }

    fn is_peektok(&self, tok: &TokenType) -> bool {
        self.peektok.ttype == *tok
    }

    const fn peek_prec(&self) -> usize {
        get_precedences(&self.peektok.ttype)
    }

    //fn cur_prec(&self) -> usize {
    //    get_precedences(&self.curtok.ttype)
    //}

    fn peek(&mut self, tok: &TokenType) -> bool {
        if self.is_peektok(tok) {
            self.next_token();
            true
        } else {
            self.peek_error(tok);
            false
        }
    }

    fn skip(&mut self, tok: &TokenType) -> bool {
        if self.is_curtok(tok) {
            self.next_token();
            return true;
        }

        false
    }

    fn peek_error(&mut self, tok: &TokenType) {
        self.errors.push(Error {
            msg: format!(
                "{:?} | Expected {:?} but got {:?}",
                self.curtok, tok, self.peektok.ttype
            ),
        });
    }

    fn got_error_jump(&mut self, msg: String) {
        self.errors.push(Error { msg });
        self.next_token();
    }

    //
    //
    // Entry Point
    //
    //

    pub fn parse_program(&mut self) -> ast::Program {
        let stms = self.parse_stmts();
        Program { stmts: stms }
    }

    //
    //
    // Parse Statments
    //
    //

    fn parse_stmts(&mut self) -> Vec<Rc<ast::Stmt>> {
        let mut stmts: Vec<Rc<ast::Stmt>> = Vec::new();
        while !self.is_curtok(&TokenType::Eof) {
            stmts.push(self.parse_single_stmt());
            self.next_token();
        }
        stmts
    }

    fn parse_single_stmt(&mut self) -> Rc<ast::Stmt> {
        match self.curtok.ttype {
            TokenType::Let => self.parse_let_stmt(),
            TokenType::Show => self.parse_show_stmt(),
            TokenType::Return => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Rc<Stmt> {
        let ctok = self.curtok.clone();
        self.next_token();

        let id = self.parse_as_identifier();

        if self.peek(&TokenType::Eq) {
            self.next_token();
        }

        let val = self.parse_expr(P_LOWEST);

        if self.is_peektok(&TokenType::Semicolon) {
            self.next_token();
        }

        Rc::new(ast::Stmt::LetStmt {
            token: ctok,
            name: id,
            value: val,
        })
    }

    fn parse_show_stmt(&mut self) -> Rc<ast::Stmt> {
        let ctok = self.curtok.clone();
        self.next_token();
        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }
        let exprs = self.parse_expr_list(&TokenType::Rparen);
        //println!("expr->{:?}", exprs);

        Rc::new(ast::Stmt::ShowStmt {
            token: ctok,
            value: exprs,
        })
    }

    fn parse_return_stmt(&mut self) -> Rc<ast::Stmt> {
        let ctok = self.curtok.clone();

        self.next_token();

        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }

        let expr = self.parse_expr(P_LOWEST);

        if self.is_peektok(&TokenType::Rparen) {
            self.next_token();
        }

        Rc::new(Stmt::ReturnStmt {
            token: ctok,
            rval: expr,
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

        ex
    }

    fn parse_block_stms(&mut self, end: &TokenType) -> Rc<Stmt> {
        let ct = self.curtok.clone();
        let mut stmts: Vec<Rc<ast::Stmt>> = Vec::new();
        while !self.is_curtok(end) && !self.is_curtok(&TokenType::Eof) {
            let s = &self.parse_single_stmt();
            stmts.push(s.clone());
            self.next_token();
        }

        Rc::new(Stmt::BlockStmt { token: ct, stmts })
    }

    //
    //
    // Parse Expressions
    //
    //

    fn parse_prefix_expr(&mut self) -> Rc<ast::Expr> {
        match self.curtok.ttype {
            TokenType::Ident => self.parse_identifier(),
            TokenType::Number => self.parse_number(),
            TokenType::String => self.parse_string_lit(),
            TokenType::True | TokenType::False => self.parse_bool(),
            TokenType::LSBracket => self.parse_array_expr(),
            TokenType::One => self.parse_func_expr(),
            TokenType::Break => self.parse_break(),
            TokenType::Include => self.parse_include_expr(),
            TokenType::If => self.parse_if_else_expr(),
            TokenType::While => self.parse_while_expr(),
            TokenType::Lparen => self.parse_grouped_expr(),
            TokenType::BANG | TokenType::Minus => {
                let op = self.curtok.clone();
                self.next_token();
                let r = self.parse_expr(P_PREFIX);
                Rc::new(ast::Expr::PrefixExpr {
                    token: op.clone(),
                    op,
                    right: r,
                })
            }
            TokenType::Lbrace => self.parse_hash_expr(),
            _ => {
                //println!("self.curtok => {:?}", self.curtok);
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
            | TokenType::LTE
            | TokenType::GT
            | TokenType::GTE
            | TokenType::And
            | TokenType::Or
            | TokenType::MOD => Ok(Rc::new(self.parse_infix_op(left))),
            TokenType::Lparen => Ok(self.parse_call_expr(left)),
            TokenType::LSBracket => Ok(self.parse_index_expr(left)),

            _ => Err(true),
        }
    }

    fn parse_infix_op(&mut self, left: Rc<ast::Expr>) -> ast::Expr {
        let op = self.curtok.clone();
        let prec = get_precedences(&op.ttype);
        self.next_token();
        let right = self.parse_expr(prec);

        ast::Expr::InfixExpr {
            token: op.clone(),
            left,
            op,
            right,
        }
    }
    fn parse_expr(&mut self, prec: usize) -> Rc<ast::Expr> {
        let mut left_expr = self.parse_prefix_expr();

        while !self.is_peektok(&TokenType::Semicolon) && prec < self.peek_prec() {
            self.next_token();
            let infx = self.parse_infix_expr(left_expr.clone());
            if let Ok(ix) = infx {
                left_expr = ix;
            } else {
                return left_expr;
            }
        }

        left_expr
    }

    fn parse_hash_expr(&mut self) -> Rc<ast::Expr> {
        let mut hash_pairs: Vec<(Rc<ast::Expr>, Rc<ast::Expr>)> = Vec::new();

        let curtok = self.curtok.clone();
        self.next_token();

        while !self.is_curtok(&TokenType::Rbrace) {
            let k = self.parse_expr(P_LOWEST);
            self.next_token();

            if self.is_curtok(&TokenType::Colon) {
                self.next_token();
            }
            let v = self.parse_expr(P_LOWEST);
            self.next_token();
            hash_pairs.push((k, v));

            if !self.skip(&TokenType::Comma) {
                break;
            }
        }

        Rc::new(ast::Expr::HashExpr {
            token: curtok,
            pairs: hash_pairs,
        })
    }

    fn parse_index_expr(&mut self, arr: Rc<ast::Expr>) -> Rc<ast::Expr> {
        let curtok = self.curtok.clone();
        self.next_token();
        let indx = self.parse_expr(P_LOWEST);
        self.next_token();
        Rc::new(ast::Expr::IndexExpr {
            token: curtok,
            left: arr,
            index: indx,
        })

        //println!("{:?}" , x);
        //x
    }

    fn parse_expr_list(&mut self, end: &TokenType) -> Vec<Rc<ast::Expr>> {
        let mut el: Vec<Rc<ast::Expr>> = Vec::new();
        if !self.is_curtok(end) {
            el.push(self.parse_expr(P_LOWEST));
        }
        if self.is_peektok(end) {
            self.next_token();
            return el;
        }

        while self.is_peektok(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            el.push(self.parse_expr(P_LOWEST))
        }

        if self.is_peektok(end) {
            self.next_token();
        }

        el
    }

    fn parse_grouped_expr(&mut self) -> Rc<ast::Expr> {
        self.next_token();
        let exp = self.parse_expr(P_LOWEST);
        self.next_token();
        exp
    }

    fn parse_if_else_expr(&mut self) -> Rc<ast::Expr> {
        let curtok = self.curtok.clone();

        self.next_token();

        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }

        let cond_expr = self.parse_expr(P_LOWEST);

        if self.is_peektok(&TokenType::Rparen) {
            self.next_token();
            self.next_token();
        }

        if self.is_curtok(&TokenType::Then) {
            self.next_token();
        }

        let true_block = self.parse_block_stms(&TokenType::Else);
        let mut else_block: Option<Rc<Stmt>> = None;

        if self.is_curtok(&TokenType::Else) {
            self.next_token();
        }

        if !self.is_curtok(&TokenType::End) {
            else_block = Some(self.parse_block_stms(&TokenType::End))
        }

        Rc::new(ast::Expr::IfExpr {
            token: curtok,
            cond: cond_expr,
            trueblock: true_block,
            elseblock: else_block,
        })
    }

    fn parse_while_expr(&mut self) -> Rc<ast::Expr> {
        let ctok = self.curtok.clone();
        self.next_token();
        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }

        let cond_expr = self.parse_expr(P_LOWEST);

        if self.is_peektok(&TokenType::Rparen) {
            self.next_token();
            self.next_token();
        }
        let loop_block = self.parse_block_stms(&TokenType::End);

        Rc::new(ast::Expr::WhileExpr {
            token: ctok,
            cond: cond_expr,
            stmts: loop_block,
        })
    }

    fn parse_func_expr(&mut self) -> Rc<ast::Expr> {
        let ct = self.curtok.clone();
        self.next_token();

        if self.is_curtok(&TokenType::Func) {
            self.next_token();
        }
        let p = self.parse_func_params();
        let bs = self.parse_block_stms(&TokenType::End);

        Rc::new(ast::Expr::FuncExpr {
            token: ct,
            params: p,
            body: bs,
        })
    }

    fn parse_func_params(&mut self) -> Rc<Vec<ast::Identifier>> {
        let mut params: Vec<ast::Identifier> = Vec::new();
        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }
        if self.is_curtok(&TokenType::Rparen) {
            self.next_token();
            return Rc::new(params);
        }

        params.push(self.parse_as_identifier());

        while self.is_peektok(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            params.push(self.parse_as_identifier())
        }

        //println!("{:?}" , self.curtok);
        if self.is_peektok(&TokenType::Rparen) {
            self.next_token();
            self.next_token();
        }

        Rc::new(params)
    }

    fn parse_call_expr(&mut self, func: Rc<ast::Expr>) -> Rc<ast::Expr> {
        let cutok = self.curtok.clone();
        self.next_token();

        let args = self.parse_expr_list(&TokenType::Rparen);

        Rc::new(ast::Expr::CallExpr {
            token: cutok,
            func,
            args,
        })
    }

    fn parse_include_expr(&mut self) -> Rc<ast::Expr> {
        let curtok = self.curtok.clone();
        self.next_token();
        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }
        let expr = self.parse_expr(P_LOWEST);

        if self.is_peektok(&TokenType::Rparen) {
            self.next_token();
        }

        Rc::new(ast::Expr::IncludeExpr {
            token: curtok,
            filename: expr,
        })
    }

    fn parse_array_expr(&mut self) -> Rc<ast::Expr> {
        let tok = self.curtok.clone();
        self.next_token();
        let elms = self.parse_expr_list(&TokenType::RSBracket);
        //self.next_token();
        Rc::new(ast::Expr::ArrayExpr {
            token: tok,
            elems: elms,
        })
    }

    //
    //
    // Primitives
    //
    //

    fn parse_identifier(&mut self) -> Rc<ast::Expr> {
        Rc::new(ast::Expr::IdentExpr {
            token: self.curtok.clone(),
            value: self.curtok.literal.to_string(),
        })
    }

    fn parse_as_identifier(&mut self) -> ast::Identifier {
        //      let ctok = self.curtok.clone();
        //        let is_mod = ctok.literal.contains('.');

        Identifier {
            token: self.curtok.clone(),
            name: self.curtok.literal.to_string(),
            is_mod: self.curtok.literal.contains('.'),
        }
    }

    fn parse_string_lit(&mut self) -> Rc<ast::Expr> {
        Rc::new(ast::Expr::StringExpr {
            token: self.curtok.clone(),
            value: self.curtok.literal.to_string(),
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
            value: self.curtok.literal.to_string(),
        })
    }

    fn parse_number(&mut self) -> Rc<ast::Expr> {
        let curtok = self.curtok.clone();
        let curtok_lit = self.curtok.literal.clone();
        //        let nl: Vec<&str> = curtok_lit.split('.').collect();

        if curtok_lit.split('.').count() == 1 {
            let v = curtok_lit.parse::<i64>().unwrap();
            Rc::new(ast::Expr::NumExpr {
                token: curtok,
                value: token::NumberToken::Int(v),
                is_int: true,
            })
        } else {
            // Should be == 2 : TO DO -> Check
            let v = curtok_lit.parse::<f64>().unwrap();
            Rc::new(ast::Expr::NumExpr {
                token: curtok,
                value: token::NumberToken::Float(v),
                is_int: false,
            })
        }
    }
}
