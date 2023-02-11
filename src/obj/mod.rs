use std::{hash::Hash, rc::Rc};

pub mod env;
use crate::{
    ast,
    token::{self, Token},
};

use self::env::Env;

pub const HASH_OBJ: &str = "hash";
pub const NUMBER_OBJ: &str = "number";
pub const BOOL_OBJ: &str = "bool";
pub const STRING_OBJ: &str = "string";
pub const ARRAY_OBJ: &str = "array";
pub const NULL_OBJ: &str = "null";
pub const RVALUE_OBJ: &str = "rvalue";
pub const ERR_OBJ: &str = "err";
pub const BREAK_OBJ: &str = "break";
pub const FUNC_OBJ: &str = "break";
pub const INCLUDE_OBJ: &str = "include";
pub const SHOW_OBJ: &str = "show";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Number {
        token: Option<Rc<Token>>,
        value: token::NumberToken,
    },
    Bool {
        token: Option<Rc<Token>>,
        value: bool,
    },
    String {
        token: Option<Rc<Token>>,
        value: String,
    },
    Array {
        token: Option<Rc<Token>>,
        value: Vec<Rc<Object>>,
    },
    Null,
    ReturnValue {
        token: Option<Rc<Token>>,
        value: Rc<Object>,
    },
    Error {
        token: Option<Option<Rc<Token>>>,
        value: String,
    },
    Break {
        token: Option<Rc<Token>>,
        value: Rc<Object>,
    },
    Function {
        token: Option<Rc<Token>>,
        params: Vec<ast::Identifier>,
        body: Rc<ast::Stmt>,
        env: Env,
    },
    Include {
        token: Option<Rc<Token>>,
        filename: String,
    },
    Show {
        token: Option<Rc<Token>>,
        value: Vec<String>,
    },
    Hash {
        token: Option<Rc<Token>>,
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

    pub fn get_type(&self) -> &str {
        match self {
            Self::Hash { .. } => HASH_OBJ,
            Self::Null => NULL_OBJ,
            Self::String { .. } => STRING_OBJ,
            Self::Bool { .. } => BOOL_OBJ,
            Self::Show { .. } => SHOW_OBJ,
            Self::Include { .. } => INCLUDE_OBJ,
            Self::Break { .. } => BREAK_OBJ,
            Self::ReturnValue { .. } => RVALUE_OBJ,
            Self::Array { .. } => ARRAY_OBJ,

            Self::Function { .. } => FUNC_OBJ,

            Self::Number { .. } => NUMBER_OBJ,
            Self::Error { .. } => ERR_OBJ,
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
