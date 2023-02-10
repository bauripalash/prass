use std::{hash::Hash, rc::Rc};

pub mod env;
use crate::{
    ast,
    token::{self, Token},
};

use self::env::Env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Number {
        token: Rc<Token>,
        value: token::NumberToken,
    },
    Bool {
        token: Rc<Token>,
        value: bool,
    },
    String {
        token: Rc<Token>,
        value: String,
    },
    Array {
        token: Rc<Token>,
        value: Vec<Rc<Object>>,
    },
    Null,
    ReturnValue {
        token: Rc<Token>,
        value: Rc<Object>,
    },
    Error {
        token: Option<Rc<Token>>,
        value: String,
    },
    Break {
        token: Rc<Token>,
        value: Rc<Object>,
    },
    Function {
        token: Rc<Token>,
        params: Vec<ast::Identifier>,
        body: Rc<ast::Stmt>,
        env: Env,
    },
    Include {
        token: Rc<Token>,
        filename: String,
    },
    Show {
        token: Rc<Token>,
        value: Vec<String>,
    },
    Hash {
        token: Rc<Token>,
        pairs: Vec<(Rc<Object>, Rc<Object>)>,
    },
}

impl Object {
    pub fn hashable(&self) -> bool {
        match self {
            Object::Number { token: _, value: _ }
            | Object::Bool { token: _, value: _ }
            | Object::String { token: _, value: _ } => true,
            _ => false,
        }
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::String { token: _, value } => value.hash(state),
            Object::Bool { token: _, value } => value.hash(state),
            Self::Number { token: _, value } => value.hash(state),
            _ => panic!("not hashable"),
        }
    }
}
