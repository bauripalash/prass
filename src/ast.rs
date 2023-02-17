use std::{fmt::Display, rc::Rc};

use crate::{
    errorhelper::ParserError,
    token::{NumberToken, Token},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Program(Program),
    Stmt(Stmt),
    Expr(Rc<Expr>),
    Identifier(Identifier),
}

pub trait AstNode {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub stmts: Vec<Rc<Stmt>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    pub token: Rc<Token>,
    pub name: String,
    pub is_mod: bool,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        for s in &self.stmts {
            res.push_str(format!("{s};").as_str())
        }
        write!(f, "PROG[{res}]")
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id<{}|{}>", self.name, self.is_mod)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord)]
pub enum Expr {
    NumExpr {
        token: Rc<Token>,
        value: NumberToken,
        is_int: bool,
    },
    IdentExpr {
        token: Rc<Token>,
        value: String,
    },
    BoolExpr {
        token: Rc<Token>,
        value: bool,
    },
    StringExpr {
        token: Rc<Token>,
        value: String,
    },
    Break {
        token: Rc<Token>,
        value: String,
    },
    PrefixExpr {
        token: Rc<Token>,
        op: Rc<Token>,
        right: Rc<Expr>,
    },

    InfixExpr {
        token: Rc<Token>,
        left: Rc<Expr>,
        op: Rc<Token>,
        right: Rc<Expr>,
    },

    ArrayExpr {
        token: Rc<Token>,
        elems: Vec<Rc<Expr>>,
    },
    IndexExpr {
        token: Rc<Token>,
        left: Rc<Expr>,
        index: Rc<Expr>,
    },

    IncludeExpr {
        token: Rc<Token>,
        filename: Rc<Expr>,
    },
    IfExpr {
        token: Rc<Token>,
        cond: Rc<Expr>,
        trueblock: Rc<Stmt>,
        elseblock: Option<Rc<Stmt>>,
    },
    WhileExpr {
        token: Rc<Token>,
        cond: Rc<Expr>,
        stmts: Rc<Stmt>, //Block Stmt
    },

    FuncExpr(FuncExpr),

    CallExpr {
        token: Rc<Token>,
        func: Rc<Expr>,
        args: Vec<Rc<Expr>>,
    },
    HashExpr {
        token: Rc<Token>,
        pairs: Vec<(Rc<Expr>, Rc<Expr>)>,
    },
    NullExpr,
    ErrExpr(ParserError),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FuncExpr {
    pub token: Rc<Token>,
    pub name: String,
    pub params: Rc<Vec<Identifier>>,
    pub body: Rc<Stmt>,
}

impl Expr {
    pub fn get_fn(&mut self) -> Option<&mut FuncExpr> {
        if let Self::FuncExpr(f) = self {
            return Some(f);
        }
        None
    }

    pub const fn is_error(&self) -> bool {
        matches!(self, Self::ErrExpr(..))
    }
}

impl AstNode for Expr {}
impl Eq for Expr {}
impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: String = match self {
            Self::NumExpr {
                token: _,
                value,
                is_int: _,
            } => match value {
                NumberToken::Int(i) => format!("({i})"),
                NumberToken::Float(f) => format!("({f})"),
            },
            Self::IdentExpr { token: _, value } => format!("ident({value})"),
            Self::BoolExpr { token: _, value } => format!("bool({value})"),
            Self::StringExpr { token: _, value } => format!("str({value})"),
            Self::Break { token: _, value: _ } => "break()".to_string(),
            Self::PrefixExpr {
                token: _,
                op,
                right,
            } => format!("pre({}{})", op.literal, right),
            Self::InfixExpr {
                token: _,
                left,
                op,
                right,
            } => format!("inf({}{}{})", left, op.literal, right),
            Self::ArrayExpr { token: _, elems } => {
                let mut arrs: String = String::new();
                for e in elems {
                    arrs.push_str(format!("{e}").as_str());
                }
                format!("arr({arrs})")
            }
            Self::IndexExpr {
                token: _,
                left,
                index,
            } => {
                format!("index({left}:{index})")
            }
            Self::IfExpr {
                token: _,
                cond,
                trueblock,
                elseblock,
            } => elseblock.as_ref().map_or_else(
                || format!("if({cond}:{trueblock})"),
                |eb| format!("if({cond}:{trueblock}:{eb})"),
            ),

            Self::WhileExpr {
                token: _,
                cond,
                stmts,
            } => {
                format!("while({cond}:{stmts})")
            }

            Self::FuncExpr(f) => {
                let mut ps = String::new();

                for p in f.params.iter() {
                    ps.push_str(format!("{p}").as_str())
                }

                format!("func({ps}:{})", f.body)
            }
            Self::IncludeExpr { token: _, filename } => {
                format!("inc({filename})")
            }

            Self::CallExpr {
                token: _,
                func,
                args,
            } => {
                let mut ar = String::new();
                for a in args {
                    ar.push_str(format!("{a},").as_str());
                }

                format!("call({func}:{ar})")
            }

            Self::HashExpr { token: _, pairs } => {
                let mut hp = String::new();

                for (a, b) in pairs {
                    hp.push_str(format!("[{a}:{b}]").as_str());
                }

                format!("hash({hp})")
            }

            Self::ErrExpr(e) => format!("{e:?}"),
            Self::NullExpr => "null".to_string(),
        };

        write!(f, "{result}")
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord)]
pub enum Stmt {
    LetStmt {
        token: Rc<Token>,
        name: Identifier,
        value: Rc<Expr>,
    },

    ReturnStmt {
        token: Rc<Token>,
        rval: Rc<Expr>,
    },

    ShowStmt {
        token: Rc<Token>,
        value: Vec<Rc<Expr>>,
    },

    BlockStmt {
        token: Rc<Token>,
        stmts: Vec<Rc<Stmt>>,
    },

    ExprStmt {
        token: Rc<Token>,
        expr: Rc<Expr>,
    },
}

impl AstNode for Stmt {}
impl Eq for Stmt {}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: String = match self {
            Self::LetStmt {
                token: _,
                name,
                value,
            } => {
                format!("let<{name}:{value}>")
            }
            Self::ReturnStmt { token: _, rval } => {
                format!("ret<{rval}>")
            }

            Self::ShowStmt { token: _, value } => {
                let mut res = String::new();
                for v in value {
                    res.push_str(format!("{v},").as_str())
                }
                format!("show<{res}>")
            }

            Self::BlockStmt { token: _, stmts } => {
                let mut res = String::new();

                for s in stmts {
                    res.push_str(format!("{s};").as_str());
                }

                format!("blk<{res}>")
            }

            Self::ExprStmt { token: _, expr } => {
                format!("{expr}")
            }
        };

        write!(f, "{result}")
    }
}
