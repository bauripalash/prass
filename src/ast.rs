use std::rc::Rc;

use crate::token::{NumberToken, Token};

pub enum Node {
    Program(Program),
    Stmt(Stmt),
    Expr(Rc<Expr>),
    Identifier(Identifier),
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Rc<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub token: Token,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    NumExpr {
        token: Token,
        value: NumberToken,
        is_int: bool,
    },
    IdentExpr {
        token: Token,
        value: String,
    },
    BoolExpr {
        token: Token,
        value: bool,
    },
    StringExpr {
        token: Token,
        value: String,
    },
    Break {
        token: Token,
        value: String,
    },
    PrefixExpr {
        token: Token,
        op: Token,
        right: Rc<Expr>,
    },

    InfixExpr {
        token: Token,
        left: Rc<Expr>,
        op: Token,
        right: Rc<Expr>,
    },

    ArrayExpr {
        token: Token,
        elems: Vec<Rc<Expr>>,
    },
    IndexExpr {
        token: Token,
        left: Rc<Expr>,
        index: Rc<Expr>,
    },
    IfExpr {
        token: Token,
        cond: Rc<Expr>,
        trueblock: Rc<Expr>,
        elseblock: Option<Rc<Expr>>,
    },
    WhileExpr {
        token: Token,
        cond: Rc<Expr>,
        stmts: Rc<Stmt>, //Block Stmt
    },

    FuncExpr {
        token: Token,
        params: Rc<Vec<Identifier>>,
        body: Rc<Stmt>,
    },

    CallExpr {
        token: Token,
        func: Rc<Expr>,
        args: Vec<Rc<Expr>>,
    },
    HashExpr {
        token: Token,
        pairs: Vec<(Rc<Expr>, Rc<Expr>)>,
    },
    ErrExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetStmt {
        token: Token,
        name: Identifier,
        value: Rc<Expr>,
    },

    ReturnStmt {
        token: Token,
        rval: Rc<Expr>,
    },

    ShowStmt {
        token: Token,
        value: Vec<Rc<Expr>>,
    },

    IncludeStmt {
        token: Token,
        filename: Rc<Expr>,
    },

    BlockStmt {
        token: Token,
        stmts: Vec<Rc<Stmt>>,
    },

    ExprStmt {
        token: Token,
        expr: Rc<Expr>,
    },
}
