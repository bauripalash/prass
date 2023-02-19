use std::fmt::Display;
use std::rc::Rc;

use crate::{compiler::code, obj::Closure};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub cl: Rc<Closure>,
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
            cl: Closure::default().into(),
            ip: -1,
            bp: 0,
        }
    }
}

impl Frame {
    pub const fn new(cf: Rc<Closure>, bp: i64) -> Self {
        Self { cl: cf, ip: -1, bp }
    }

    pub fn get_instructions(&self) -> Rc<code::Instructions> {
        self.cl.fun.fnin.clone()
    }

    pub fn get_ins_len(&self) -> i64 {
        self.cl.fun.fnin.ins.len() as i64
    }

    pub fn get_ip(&self) -> i64 {
        self.ip
    }
}
