use std::rc::Rc;
use std::{cell::RefCell, fmt::Display};

use crate::{compiler::code, obj::Closure};

static FRAMES_SIZE: usize = 1024;
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
        Rc::clone(&self.cl.fun.fnin)
    }

    pub fn get_ins_len(&self) -> i64 {
        self.cl.fun.fnin.ins.len() as i64
    }

    pub fn get_ip(&self) -> i64 {
        self.ip
    }

    pub fn set_ip(&mut self, ip: i64) {
        self.ip = ip;
    }

    pub fn adv_ip(&mut self, ip: i64) {
        self.ip += ip
    }

    pub fn get_bp(&self) -> i64 {
        self.bp
    }

    pub fn set_bp(&mut self, bp: i64) {
        self.bp = bp
    }

    pub fn adv_bp(&mut self, bp: i64) {
        self.bp += bp
    }

    pub fn get_cl(&self) -> Rc<Closure> {
        Rc::clone(&self.cl)
    }
}

#[derive(Debug)]
pub struct FramePool {
    pub frames: Vec<Rc<RefCell<Frame>>>,
    pub len: usize,
}

impl FramePool {
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(FRAMES_SIZE), //Rc::new(RefCell::new(Vec::with_capacity(FRAMES_SIZE))),
            len: 0,
        }
    }
}
