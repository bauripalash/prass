use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt::Display,
    hash::{Hash, Hasher},
    rc::Rc,
};

pub mod env;
use crate::{
    ast,
    compiler::code::Instructions,
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
pub const FUNC_OBJ: &str = "func";
pub const INCLUDE_OBJ: &str = "include";
pub const SHOW_OBJ: &str = "show";
pub const COMPILED_FUNC_OBJ: &str = "compiled_func";
pub const CLOSURE_OBJ: &str = "closure";

#[derive(Debug, Clone)]
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
        pairs: HashMap<Rc<HashKey>, Rc<HashPair>>,
    },

    Compfunc(CompFunc),

    Closure(Closure),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        match self {
            Self::Number { token: _, value } => result.push_str(&value.to_string()),
            Self::String { token: _, value } => result.push_str(&value),
            Self::Bool { token: _, value } => result.push_str(&value.to_string()),
            Self::Array { token: _, value } => {
                for item in value.iter() {
                    result.push_str(&(item.to_string() + " "))
                }
            }
            Self::Null => result.push_str("null"),
            Self::Hash { token: _, pairs } => {
                //println!("{:?}" , pairs);
                //for p in pairs.values(){
                for (_, v) in pairs.iter() {
                    result.push_str(format!("{}, ", v).as_str())
                    //result.push_str(format!("{}:{},", p.key , p.value).as_str())
                }
            }
            _ => {}
        }
        write!(f, "{result}")
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Closure {
    pub fun: CompFunc,
    pub frees: Vec<Rc<Object>>,
}

impl Closure {
    pub const fn new(fnin: Instructions) -> Self {
        Self {
            fun: CompFunc::new(fnin),
            frees: Vec::new(),
        }
    }

    pub const fn new_from_cfn(fun: CompFunc) -> Self {
        Self { fun, frees: vec![] }
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CL({}->{})", self.fun, self.frees.len())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompFunc {
    pub fnin: Instructions,
    pub num_locals: usize,
    pub num_params: usize,
}

impl Display for CompFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FN({})", self.fnin)
    }
}

impl Default for CompFunc {
    fn default() -> Self {
        Self {
            fnin: Instructions::new(),
            num_locals: 0,
            num_params: 0,
        }
    }
}

impl CompFunc {
    pub const fn new(fnin: Instructions) -> Self {
        Self {
            fnin,
            num_locals: 0,
            num_params: 0,
        }
    }
}
/*
impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closure(cl) => write!(f, "cl({cl})"),
            Self::Compfunc(cfn) => write!(f, "fn({cfn})"),
            _ => write!(f, "{self:?}"),
        }
    }
}
*/

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct HashKey {
    pub key: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashPair {
    pub key: Rc<Object>,
    pub value: Rc<Object>,
}

impl Display for HashPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.key, self.value)
    }
}

impl Object {
    pub const fn hashable(&self) -> bool {
        matches!(
            self,
            Self::Number { token: _, value: _ }
                | Self::Bool { token: _, value: _ }
                | Self::String { token: _, value: _ }
        )
    }

    pub fn get_hash(&self) -> u64 {
        if !self.hashable() {
            panic!("not hashable")
        }

        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub const fn get_type(&self) -> &str {
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
            Self::Compfunc { .. } => COMPILED_FUNC_OBJ,
            Self::Closure { .. } => CLOSURE_OBJ,
        }
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::String { token: _, value } => value.hash(state),
            Self::Bool { token: _, value } => value.hash(state),
            Self::Number { token: _, value } => value.hash(state),
            _ => panic!("not hashable"),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Eq for Object {}
