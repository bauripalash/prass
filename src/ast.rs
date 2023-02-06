use std::rc::Rc;

use crate::token::{NumberToken, Token};

pub enum Node {
    Program(Program),
    Stmt(Stmt),
    Expr(Rc<Expr>),
}

pub struct Program {
    pub stmts: Vec<Rc<Stmt>>,
}

pub struct Identifier {
    pub token: Token,
    pub name: String,
}

pub enum Expr {
    NumExpr { token: Token, value: NumberToken },
    BoolExpr { token: Token, value: bool },
}

pub enum Stmt {
    LetStmt {
        token: Token,
        identifier: Identifier,
        value: Rc<Expr>,
    },
}
