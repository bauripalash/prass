use std::rc::Rc;

use crate::{
    ast::{self, Expr, Identifier, Program, Stmt},
    errorhelper::ParserError,
    lexer::Lexer,
    token::{self, Token, TokenType},
};

type ParseResult<T> = std::result::Result<T, ParserError>;

//#[allow(dead_code)]
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
pub struct Parser<'pax> {
    lexer: Lexer<'pax>,
    curtok: Rc<Token>,
    peektok: Rc<Token>,
    pub errors: Vec<ParserError>,
}

impl<'pax> Parser<'pax> {
    pub fn new(lexer: Lexer<'pax>) -> Self {
        let mut p = Self {
            lexer,
            curtok: Rc::new(Token::dummy()),
            peektok: Rc::new(Token::dummy()),
            errors: vec![],
        };

        p.next_token();
        p.next_token();
        p
    }

    fn last_error(&self) -> &ParserError {
        self.errors.last().unwrap()
    }

    //
    //
    //  Helper Functions
    //
    //

    pub fn print_errorrs(&self) {
        let errh = &self.lexer.eh;

        for e in &self.errors {
            if let Some(et) = &e.token {
                println!("{}", errh.show_error(et));

                println!("{} -> {}", e.msg, et.literal);
            } else {
                println!("{}", e.msg);
            }
        }
    }

    fn next_token(&mut self) -> Rc<Token> {
        if self.peektok.ttype == TokenType::Illegal {
            self.errors.push(ParserError::new(
                "Illegal token",
                Some(&self.peektok),
                Some(&self.peektok.ttype),
            ));

            //self.next_token();
        }
        self.curtok = self.peektok.clone();
        if let Ok(nt) = self.lexer.next_token() {
            self.peektok = nt;
        } else {
            panic!("lexer error->")
        }
        self.curtok.clone()
    }

    fn is_curtok(&self, tok: &TokenType) -> bool {
        self.curtok.ttype == *tok
    }

    fn is_peektok(&self, tok: &TokenType) -> bool {
        self.peektok.ttype == *tok
    }

    fn peek_prec(&self) -> usize {
        get_precedences(&self.peektok.ttype)
    }

    fn expect(&mut self, tok: &TokenType) -> bool {
        if self.peek(tok) {
            self.next_token();
            true
        } else {
            false
        }
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

    fn now(&mut self, tok: &TokenType) -> bool {
        if self.is_curtok(tok) {
            self.next_token();
            true
        } else {
            let errormsg = format!("Expected {:?} but got {:?}", tok, self.curtok);
            self.errors.push(ParserError::new(
                &errormsg,
                Some(&self.curtok),
                Some(&self.curtok.ttype),
            ));
            false
        }
    }

    fn peek_error(&mut self, tok: &TokenType) {
        println!("{}", self.lexer.eh.show_error(&self.peektok));
        let errmsg = format!(
            "{:?} | Expected {:?} but got {:?}",
            self.curtok, tok, self.peektok.ttype
        );
        self.errors.push(ParserError::new(
            &errmsg,
            Some(&self.peektok),
            Some(&self.peektok.ttype),
        ));
    }

    /*fn got_error_jump(&mut self, msg: String) {
        self.errors.push(Error::new(&msg, None, None));
        self.next_token();
    }*/
    fn got_error_jump_with_err(&mut self, err: ParserError) {
        self.errors.push(err);
        self.next_token();
    }

    fn skip_semicolon(&mut self) -> bool {
        if self.is_peektok(&TokenType::Semicolon) {
            self.next_token();
            return true;
        }
        false
    }

    //
    //
    // Entry Point
    //
    //

    pub fn parse_program(&mut self) -> ParseResult<ast::Program> {
        if let Ok(stmts) = self.parse_stmts() {
            return Ok(Program { stmts });
        }
        Err(self.errors[0].clone())
        //return self.parse_stmts()

        //Ok(Program { stmts: stms })
    }

    //
    //
    // Parse Statments
    //
    //

    fn parse_stmts(&mut self) -> ParseResult<Vec<Rc<ast::Stmt>>> {
        let mut stmts: Vec<Rc<ast::Stmt>> = Vec::new();
        while !self.is_curtok(&TokenType::Eof) {
            let s = self.parse_single_stmt();

            if let Ok(ss) = s {
                stmts.push(ss);
                self.next_token();
            } else {
                return Err(s.err().unwrap());
            }
        }
        Ok(stmts)
    }

    fn parse_single_stmt(&mut self) -> ParseResult<Rc<ast::Stmt>> {
        match self.curtok.ttype {
            TokenType::Let => self.parse_let_stmt(),
            TokenType::Show => self.parse_show_stmt(),
            TokenType::Return => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> ParseResult<Rc<Stmt>> {
        let ctok = self.curtok.clone();
        if !self.now(&TokenType::Let) {
            return Err(self.last_error().to_owned());
        }

        //self.next_token();

        let id = self.parse_as_identifier().unwrap(); //Note: It is safe for
                                                      //now to unwrap
                                                      //self.now(&TokenType::Ident);
                                                      //println!("{:?} {:?}" , self.curtok ,self.peektok);
        if self.peek(&TokenType::Eq) {
            self.next_token();
        } else {
            return Err(self.last_error().to_owned());
        }

        let raw_expr_val = self.parse_expr(P_LOWEST);
        let Ok(mut expr_val) = raw_expr_val else{
            return Err(raw_expr_val.err().unwrap());
        };
        //let mut expr_val = self.parse_expr(P_LOWEST);
        let mut func_binding = expr_val.as_ref().clone();
        let func_val = func_binding.get_fn();

        if let Some(f) = func_val {
            f.name = id.name.clone();

            expr_val = Rc::new(Expr::FuncExpr(f.clone()))
        }

        self.skip_semicolon();

        Ok(Rc::new(ast::Stmt::LetStmt {
            token: ctok,
            name: id,
            value: expr_val,
        }))
    }

    fn parse_show_stmt(&mut self) -> ParseResult<Rc<ast::Stmt>> {
        let ctok = self.curtok.clone();
        if !self.now(&TokenType::Show) {
            return Err(self.last_error().to_owned());
        }

        if !self.now(&TokenType::Lparen) {
            return Err(self.last_error().to_owned());
        }
        let raw_exprs = self.parse_expr_list(&TokenType::Rparen);
        let Ok(value) = raw_exprs else {
            return Err(raw_exprs.err().unwrap());
        };

        self.skip_semicolon();
        Ok(Rc::new(ast::Stmt::ShowStmt { token: ctok, value }))
    }

    fn parse_return_stmt(&mut self) -> ParseResult<Rc<ast::Stmt>> {
        let ctok = self.curtok.clone();
        if !self.now(&TokenType::Return) {
            return Err(self.last_error().to_owned());
        }
        if !self.now(&TokenType::Lparen) {
            return Err(self.last_error().to_owned());
        }
        let raw_expr = self.parse_expr(P_LOWEST);
        let Ok(rval) = raw_expr else{
            return Err(raw_expr.err().unwrap());
        };
        if !self.peek(&TokenType::Rparen) {
            return Err(self.last_error().to_owned());
        }
        self.skip_semicolon();
        Ok(Rc::new(Stmt::ReturnStmt { token: ctok, rval }))
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<Rc<Stmt>> {
        let token = self.curtok.clone();
        let raw_expr = self.parse_expr(P_LOWEST);
        let Ok(expr) = raw_expr else{
            return Err(raw_expr.err().unwrap());
        };
        let ex = Rc::new(ast::Stmt::ExprStmt { token, expr });

        self.skip_semicolon();

        Ok(ex)
    }

    fn parse_block_stms(&mut self, end: &TokenType) -> ParseResult<Rc<Stmt>> {
        let ct = self.curtok.clone();
        let mut stmts: Vec<Rc<ast::Stmt>> = Vec::new();
        while !self.is_curtok(end) && !self.is_curtok(&TokenType::Eof) {
            let s = self.parse_single_stmt();
            if let Ok(ss) = s {
                stmts.push(ss);
                self.next_token();
            } else {
                return s;
            }
        }

        Ok(Rc::new(Stmt::BlockStmt { token: ct, stmts }))
    }

    //
    //
    // Parse Expressions
    //
    //

    fn parse_prefix_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
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
                let raw_r = self.parse_expr(P_PREFIX);
                let Ok(r) = raw_r else{
                return Err(raw_r.err().unwrap());
                };

                Ok(Rc::new(ast::Expr::PrefixExpr {
                    token: op.clone(),
                    op,
                    right: r,
                }))
            }
            TokenType::Lbrace => self.parse_hash_expr(),
            _ => {
                let err =
                    ParserError::new("Unknown Prefix; Unexpected Token", Some(&self.curtok), None);
                self.got_error_jump_with_err(err.clone());
                Err(err)
                //Rc::new(ast::Expr::ErrExpr(err))
            }
        }
    }

    fn parse_infix_expr(&mut self, left: Rc<ast::Expr>) -> ParseResult<Rc<ast::Expr>> {
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
            | TokenType::MOD => self.parse_infix_op(left),

            TokenType::Lparen => self.parse_call_expr(left),
            TokenType::LSBracket => self.parse_index_expr(left),

            _ => Err(ParserError::new(
                "Unknown Infix Operator",
                Some(&self.curtok),
                Some(&self.curtok.ttype),
            )),
        }
    }

    fn parse_infix_op(&mut self, left: Rc<ast::Expr>) -> ParseResult<Rc<ast::Expr>> {
        let op = self.curtok.clone();
        let prec = get_precedences(&op.ttype);
        self.next_token();
        let raw_right = self.parse_expr(prec);
        let Ok(right) = raw_right else{
            return Err(raw_right.err().unwrap());
        };

        Ok(Rc::new(ast::Expr::InfixExpr {
            token: op.clone(),
            left,
            op,
            right,
        }))
    }
    fn parse_expr(&mut self, prec: usize) -> ParseResult<Rc<ast::Expr>> {
        let raw_left_expr = self.parse_prefix_expr();
        let Ok(mut left_expr) = raw_left_expr else{
            return Err(raw_left_expr.err().unwrap());
        };

        while !self.is_peektok(&TokenType::Semicolon) && prec < self.peek_prec() {
            self.next_token();
            let infx = self.parse_infix_expr(left_expr.clone());

            //return infx;

            if let Ok(infix_expr) = infx {
                left_expr = infix_expr;
            } else {
                return Ok(left_expr);
            }
            /*if let Ok(ix) = infx {
                left_expr = Ok(ix);
            } else {
                return left_expr;
            }*/
        }

        Ok(left_expr)
    }

    fn parse_hash_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
        let mut hash_pairs: Vec<(Rc<ast::Expr>, Rc<ast::Expr>)> = Vec::new();

        let curtok = self.curtok.clone();
        self.next_token();

        while !self.is_curtok(&TokenType::Rbrace) {
            let raw_k = self.parse_expr(P_LOWEST);

            let Ok(k) = raw_k else{
                return Err(raw_k.err().unwrap());
            };
            //drop(raw_k); //TODO:
            self.next_token();

            //if self.is_curtok(&TokenType::Colon) {
            //    self.next_token();
            //}

            self.now(&TokenType::Colon);

            let raw_v = self.parse_expr(P_LOWEST);
            let Ok(v) = raw_v else{
                return Err(raw_v.err().unwrap());
            };
            self.next_token();
            hash_pairs.push((k, v));

            if !self.skip(&TokenType::Comma) {
                break;
            }
        }

        Ok(Rc::new(ast::Expr::HashExpr {
            token: curtok,
            pairs: hash_pairs,
        }))
    }

    fn parse_index_expr(&mut self, arr: Rc<ast::Expr>) -> ParseResult<Rc<ast::Expr>> {
        let curtok = self.curtok.clone();
        self.next_token();

        let raw_index = self.parse_expr(P_LOWEST);

        let Ok(index) = raw_index else{
            return Err(raw_index.err().unwrap());
        };

        //drop(raw_index); //TODO : What does this do?

        self.next_token();
        Ok(Rc::new(ast::Expr::IndexExpr {
            token: curtok,
            left: arr,
            index,
        }))

        //println!("{:?}" , x);
        //x
    }

    fn parse_expr_list(&mut self, end: &TokenType) -> ParseResult<Vec<Rc<ast::Expr>>> {
        let mut el: Vec<Rc<ast::Expr>> = Vec::new();
        if !self.is_curtok(end) {
            let raw_expr = self.parse_expr(P_LOWEST);
            let Ok(expr) = raw_expr else{
                return Err(raw_expr.err().unwrap());
            };

            el.push(expr);
        }
        if self.is_peektok(end) {
            self.next_token();
            return Ok(el);
        }

        while self.is_peektok(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let raw_expr = self.parse_expr(P_LOWEST);
            let Ok(expr) = raw_expr else{
                return Err(raw_expr.err().unwrap());
            };
            el.push(expr);
        }

        //if self.is_peektok(end) {
        //    self.next_token();
        //}
        self.peek(end);

        Ok(el)
    }

    fn parse_grouped_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
        self.next_token();
        let exp = self.parse_expr(P_LOWEST);
        self.next_token();
        exp
    }

    fn parse_if_else_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
        let curtok = self.curtok.clone();

        if !self.now(&TokenType::If) {
            return Err(self.last_error().to_owned());
        }
        if !self.now(&TokenType::Lparen) {
            return Err(self.last_error().to_owned());
        }
        let raw_cond_expr = self.parse_expr(P_LOWEST);
        let Ok(cond_expr) = raw_cond_expr else{
            return Err(raw_cond_expr.err().unwrap());
        };
        if self.peek(&TokenType::Rparen) {
            self.next_token();
        }

        if !self.now(&TokenType::Then) {
            return Err(self.last_error().to_owned());
        }

        let raw_true_block = self.parse_block_stms(&TokenType::Else);
        let Ok(true_block) = raw_true_block else{
            return Err(raw_true_block.err().unwrap());
        };
        let mut else_block: Option<Rc<Stmt>> = None;

        if !self.now(&TokenType::Else) {
            return Err(self.last_error().to_owned());
        }

        if !self.is_curtok(&TokenType::End) {
            let raw_else_block = self.parse_block_stms(&TokenType::End);
            let Ok(else_block_stmts) = raw_else_block else{
                return Err(raw_else_block.err().unwrap());
            };
            else_block = Some(else_block_stmts);
        }

        Ok(Rc::new(ast::Expr::IfExpr {
            token: curtok,
            cond: cond_expr,
            trueblock: true_block,
            elseblock: else_block,
        }))
    }

    fn parse_while_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
        let token = self.curtok.clone();
        self.next_token();
        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }

        let raw_cond_expr = self.parse_expr(P_LOWEST);
        let Ok(cond) = raw_cond_expr else {
            return Err(raw_cond_expr.err().unwrap());
        };

        if self.is_peektok(&TokenType::Rparen) {
            self.next_token();
            self.next_token();
        }
        let raw_loop_block = self.parse_block_stms(&TokenType::End);

        let Ok(stmts) = raw_loop_block else{
            return Err(raw_loop_block.err().unwrap());
        };

        Ok(Rc::new(ast::Expr::WhileExpr { token, cond, stmts }))
    }

    fn parse_func_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
        let ct = self.curtok.clone();
        self.expect(&TokenType::Func);

        // Current token is One/Ekti
        // Next token should be Func
        //
        // If next token is func;
        // we advance.
        //
        // So now  ->
        // [EKTI] [FUNC] [(] [a] [)]
        //:
        //if self.peek(&TokenType::Func) {
        //    self.next_token();
        //}
        //self.next_token();
        //println!("{:?} -> {:?}", self.curtok, self.peektok);

        //self.next_token();

        let p = self.parse_func_params();
        let Ok(params) = p else{
            return Err(p.err().unwrap());
        };

        let bs = self.parse_block_stms(&TokenType::End);

        let Ok(body) = bs else{
            return Err(bs.err().unwrap());
        };

        Ok(Rc::new(ast::Expr::FuncExpr(ast::FuncExpr {
            name: String::from(""),
            token: ct,
            params,
            body,
        })))
    }

    fn parse_func_params(&mut self) -> ParseResult<Rc<Vec<ast::Identifier>>> {
        let mut params: Vec<ast::Identifier> = Vec::new();
        self.now(&TokenType::Lparen);
        //if self.is_curtok(&TokenType::Lparen) {
        //    self.next_token();
        //}
        if self.is_curtok(&TokenType::Rparen) {
            self.next_token();
            return Ok(Rc::new(params));
        }

        let id = self.parse_as_identifier();
        if let Ok(rid) = id {
            params.push(rid)
        } else {
            return Err(ParserError::new(
                "Expected Identifier but failed to parse this as identifier",
                Some(&self.curtok),
                Some(&self.curtok.ttype),
            ));
        }
        //params.push(;

        while self.is_peektok(&TokenType::Comma) {
            self.next_token();
            self.next_token();
            let next_id = self.parse_as_identifier();
            if let Ok(rid) = next_id {
                params.push(rid)
            } else {
                return Err(ParserError::new(
                    "Expected Identifier but failed to parse this as identifier",
                    Some(&self.curtok),
                    Some(&self.curtok.ttype),
                ));
            }
            //            params.push(self.parse_as_identifier())
        }

        //println!("{:?}" , self.curtok);
        if self.peek(&TokenType::Rparen) {
            //self.next_token();
            self.next_token();
        } else {
            return Err(self.last_error().to_owned());
        }

        Ok(Rc::new(params))
    }

    fn parse_call_expr(&mut self, func: Rc<ast::Expr>) -> ParseResult<Rc<ast::Expr>> {
        let token = self.curtok.clone();
        self.next_token();

        let raw_args = self.parse_expr_list(&TokenType::Rparen);
        let Ok(args) = raw_args else {
            return Err(raw_args.err().unwrap());
        };

        Ok(Rc::new(ast::Expr::CallExpr { token, func, args }))
    }

    fn parse_include_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
        let curtok = self.curtok.clone();
        self.next_token();
        if self.is_curtok(&TokenType::Lparen) {
            self.next_token();
        }
        let raw_filename = self.parse_expr(P_LOWEST);
        let Ok(filename) = raw_filename else{
            return Err(raw_filename.err().unwrap());
        };

        if self.is_peektok(&TokenType::Rparen) {
            self.next_token();
        }

        Ok(Rc::new(ast::Expr::IncludeExpr {
            token: curtok,
            filename,
        }))
    }

    fn parse_array_expr(&mut self) -> ParseResult<Rc<ast::Expr>> {
        let token = self.curtok.clone();
        if !self.now(&TokenType::LSBracket) {
            return Err(self.last_error().to_owned());
        }
        //self.next_token();
        let raw_elms = self.parse_expr_list(&TokenType::RSBracket);
        let Ok(elems) = raw_elms else{
            return Err(raw_elms.err().unwrap());
        };
        //self.next_token();
        Ok(Rc::new(ast::Expr::ArrayExpr { token, elems }))
    }

    //
    //
    // Primitives
    //
    //

    fn parse_identifier(&mut self) -> ParseResult<Rc<ast::Expr>> {
        Ok(Rc::new(ast::Expr::IdentExpr {
            token: self.curtok.clone(),
            value: self.curtok.literal.to_string(),
        }))
    }

    fn parse_as_identifier(&mut self) -> ParseResult<ast::Identifier> {
        //      let ctok = self.curtok.clone();
        //        let is_mod = ctok.literal.contains('.');

        Ok(Identifier {
            token: self.curtok.clone(),
            name: self.curtok.literal.to_string(),
            is_mod: self.curtok.literal.contains('.'),
        })
    }

    fn parse_string_lit(&mut self) -> ParseResult<Rc<ast::Expr>> {
        Ok(Rc::new(ast::Expr::StringExpr {
            token: self.curtok.clone(),
            value: self.curtok.literal.to_string(),
        }))
    }

    fn parse_bool(&mut self) -> ParseResult<Rc<ast::Expr>> {
        Ok(Rc::new(ast::Expr::BoolExpr {
            token: self.curtok.clone(),
            value: self.is_curtok(&TokenType::True),
        }))
    }

    fn parse_break(&mut self) -> ParseResult<Rc<ast::Expr>> {
        Ok(Rc::new(ast::Expr::Break {
            token: self.curtok.clone(),
            value: self.curtok.literal.to_string(),
        }))
    }

    fn parse_number(&mut self) -> ParseResult<Rc<ast::Expr>> {
        let curtok = self.curtok.clone();
        let curtok_lit = curtok.literal.clone();
        //        let nl: Vec<&str> = curtok_lit.split('.').collect();

        if curtok_lit.split('.').count() == 1 {
            let v = curtok_lit.parse::<i64>();

            if let Ok(num) = v {
                Ok(Rc::new(ast::Expr::NumExpr {
                    token: curtok,
                    value: token::NumberToken::Int(num),
                    is_int: true,
                }))
            } else {
                //Rc::new(ast::Expr::ErrExpr(

                Err(ParserError::new(
                    "Invalid Integer number",
                    Some(&curtok),
                    Some(&curtok.ttype),
                ))

                //))
            }
        } else {
            // Should be == 2 : TO DO -> Check
            let v = curtok_lit.parse::<f64>();
            if let Ok(num) = v {
                Ok(Rc::new(ast::Expr::NumExpr {
                    token: curtok,
                    value: token::NumberToken::Float(num),
                    is_int: false,
                }))
            } else {
                //Rc::new(ast::Expr::ErrExpr(

                Err(ParserError::new(
                    "Invalid Decimal/float number",
                    Some(&curtok),
                    Some(&curtok.ttype),
                ))
                //))
            }
        }
    }
}
