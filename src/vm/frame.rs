use std::fmt::Display;

use crate::{compiler::code, obj::CompFunc};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub compfn: CompFunc,
    pub ip: i64,
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Frame[{}|ip->{}]", self.compfn, self.ip)
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            compfn: CompFunc::default(),
            ip: -1,
        }
    }
}

impl Frame {
    pub const fn new(cf: CompFunc) -> Self {
        Self { compfn: cf, ip: -1 }
    }

    pub fn get_instructions(&self) -> code::Instructions {
        self.compfn.fnin.clone()
    }
}
