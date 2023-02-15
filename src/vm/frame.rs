use std::fmt::Display;

use crate::{
    compiler::code,
    obj::Closure,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub cl: Closure,
    pub ip: i64,
    pub bp: i64,
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Frame[{}|ip->{}]", self.cl, self.ip)
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            cl: Closure::default(),
            ip: -1,
            bp: 0,
        }
    }
}

impl Frame {
    pub const fn new(cf: Closure, bp: i64) -> Self {
        Self { cl: cf, ip: -1, bp }
    }

    pub fn get_instructions(&self) -> code::Instructions {
        self.cl.fun.fnin.clone()
    }
}
