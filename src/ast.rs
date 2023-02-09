use std::{fmt::Display, rc::Rc};

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

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        for s in &self.stmts {
            res.push_str(format!("{};", s).as_str())
        }
        write!(f, "PROG[{}]", res)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id<{}>", self.name)
    }
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

    IncludeExpr {
        token: Token,
        filename: Rc<Expr>,
    },
    IfExpr {
        token: Token,
        cond: Rc<Expr>,
        trueblock: Rc<Stmt>,
        elseblock: Option<Rc<Stmt>>,
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
    NullExpr,
    ErrExpr,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: String = match self {
            Expr::NumExpr {
                token: _,
                value,
                is_int: _,
            } => match value {
                NumberToken::Int(i) => format!("({})", i),
                NumberToken::Float(f) => format!("({})", f),
            },
            Expr::IdentExpr { token: _, value } => format!("ident({})", value),
            Expr::BoolExpr { token: _, value } => format!("bool({})", value),
            Expr::StringExpr { token: _, value } => format!("str({})", value),
            Expr::Break { token: _, value: _ } => format!("break()"),
            Expr::PrefixExpr {
                token: _,
                op,
                right,
            } => format!("pre({}{})", op.literal, right),
            Expr::InfixExpr {
                token: _,
                left,
                op,
                right,
            } => format!("inf({}{}{})", left, op.literal, right),
            Expr::ArrayExpr { token: _, elems } => {
                let mut arrs: String = String::new();
                for e in elems {
                    arrs.push_str(format!("{}", e).as_str());
                }
                format!("arr({})", arrs)
            }
            Expr::IndexExpr {
                token: _,
                left,
                index,
            } => {
                format!("index({}:{})", left, index)
            }
            Expr::IfExpr {
                token: _,
                cond,
                trueblock,
                elseblock,
            } => {
                if let Some(eb) = elseblock {
                    format!("if({}:{}:{})", cond, trueblock, eb)
                } else {
                    format!("if({}:{})", cond, trueblock)
                }
            }

            Self::WhileExpr {
                token: _,
                cond,
                stmts,
            } => {
                format!("while({}:{})", cond, stmts)
            }

            Self::FuncExpr {
                token: _,
                params,
                body,
            } => {
                let mut ps = String::new();

                for p in params.to_vec() {
                    ps.push_str(format!("{}", p).as_str())
                }

                format!("func({}:{})", ps, body)
            }
            Self::IncludeExpr { token: _, filename } => {
                format!("inc({})", filename)
            }

            Self::CallExpr {
                token: _,
                func,
                args,
            } => {
                let mut ar = String::new();
                for a in args {
                    ar.push_str(format!("{},", a).as_str());
                }

                format!("call({}:{})", func, ar)
            }

            Self::HashExpr { token: _, pairs } => {
                let mut hp = String::new();

                for (a, b) in pairs {
                    hp.push_str(format!("[{}:{}]", a, b).as_str());
                }

                format!("hash({})", hp)
            }

            Self::ErrExpr => "err()".to_string(),
            Self::NullExpr => "null".to_string(),
        };

        write!(f, "{}", result)
    }
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

    BlockStmt {
        token: Token,
        stmts: Vec<Rc<Stmt>>,
    },

    ExprStmt {
        token: Token,
        expr: Rc<Expr>,
    },
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result: String = match self {
            Stmt::LetStmt {
                token: _,
                name,
                value,
            } => {
                format!("let<{}:{}>", name, value)
            }
            Self::ReturnStmt { token: _, rval } => {
                format!("ret<{}>", rval)
            }

            Self::ShowStmt { token: _, value } => {
                let mut res = String::new();
                for v in value {
                    res.push_str(format!("{},", v).as_str())
                }
                format!("show<{}>", res)
            }

            Stmt::BlockStmt { token: _, stmts } => {
                let mut res = String::new();

                for s in stmts {
                    res.push_str(format!("{};", s).as_str());
                }

                format!("blk<{}>", res)
            }

            Self::ExprStmt { token: _, expr } => {
                format!("{}", expr)
            }
        };

        write!(f, "{}", result)
    }
}
