use std::fmt::Display;
use std::io::Cursor;
use std::rc::Rc;

use byteorder::{self, ReadBytesExt, WriteBytesExt};
use byteorder::{BigEndian, ByteOrder};

use crate::obj::Object;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    Const,
    Add,
    Sub,
    Mul,
    Div,
    Pop,
    True,
    False,
    Equal,
    NotEqual,
    GT,
    JumpNotTruthy,
    Jump,
    Null,
    Bang,
    Minus,
    GetGlobal,
    SetGlobal,
    Array,
    Hash,
    Index,
    Call,
    ReturnValue,
    Return,
    GetLocal,
    SetLocal,
    Closure,
    GetFree,
    CurrentClosure,
    Dummy,
    Mod,
    Show,
}

#[allow(dead_code)]
pub struct OpDef {
    pub name: String,
    pub op_width: Vec<i64>,
}

#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Rc<Instructions>,
    pub constants: Vec<Rc<Object>>,
}

impl Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        result.push_str(&format!("Instructions:\n{}\n\nConstants:{:?}\n" , self.instructions , self.constants));
        write!(f , "{result}")
    }
}

impl OpDef {
    pub fn new(name: &str, op_width: Vec<i64>) -> Self {
        Self {
            name: name.to_string(),
            op_width,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instructions {
    pub ins: Vec<u8>,
}

pub fn get_def(op: &Opcode) -> OpDef {
    match op {
        Opcode::Const => OpDef::new("OpConst", vec![2]),
        Opcode::Add => OpDef::new("OpAdd", vec![]),
        Opcode::Sub => OpDef::new("OpSub", vec![]),
        Opcode::Mul => OpDef::new("OpMul", vec![]),
        Opcode::Div => OpDef::new("OpDiv", vec![]),
        Opcode::Pop => OpDef::new("OpPop", vec![]),
        Opcode::True => OpDef::new("OpTrue", vec![]),
        Opcode::False => OpDef::new("OpFalse", vec![]),
        Opcode::Equal => OpDef::new("OpEqual", vec![]),
        Opcode::NotEqual => OpDef::new("OpNotEqual", vec![]),
        Opcode::GT => OpDef::new("OpGT", vec![]),
        Opcode::JumpNotTruthy => OpDef::new("OpJumpNotTruthy", vec![2]),
        Opcode::Jump => OpDef::new("OpJump", vec![2]),
        Opcode::Null => OpDef::new("OpNull", vec![]),
        Opcode::Minus => OpDef::new("OpMinus", vec![]),
        Opcode::Bang => OpDef::new("OpBang", vec![]),
        Opcode::GetGlobal => OpDef::new("OpGetGlobal", vec![2]),
        Opcode::SetGlobal => OpDef::new("OpSetGlobal", vec![2]),
        Opcode::Array => OpDef::new("OpArray", vec![2]),
        Opcode::Hash => OpDef::new("OpHash", vec![2]),
        Opcode::Index => OpDef::new("OpIndex", vec![]),
        Opcode::Call => OpDef::new("OpCall", vec![1]),
        Opcode::ReturnValue => OpDef::new("OpReturnValue", vec![]),
        Opcode::Return => OpDef::new("OpReturn", vec![]),
        Opcode::GetLocal => OpDef::new("OpGetLocal", vec![1]),
        Opcode::SetLocal => OpDef::new("OpSetLocal", vec![1]),
        Opcode::Closure => OpDef::new("OpClosure", vec![2, 1]),
        Opcode::GetFree => OpDef::new("OpGetFree", vec![1]),
        Opcode::CurrentClosure => OpDef::new("OpCurrentClosure", vec![]),
        Opcode::Dummy => OpDef::new("OpDummy", vec![]),
        Opcode::Mod => OpDef::new("OpMod", vec![]),
        Opcode::Show => OpDef::new("OpShow", vec![1]),
    }
}

pub fn make_ins(op: Opcode, ops: &[usize]) -> Vec<u8> {
    let mut ins: Vec<u8> = Vec::new();
    ins.push(op as u8);
    let def = get_def(&op).op_width;
    for (o, w) in ops.iter().zip(def) {
        match w {
            2 => ins.write_u16::<BigEndian>(*o as u16).unwrap(),
            1 => ins.write_u8(*o as u8).unwrap(),
            _ => panic!("unsupported op width"),
        }
    }

    ins
}

pub fn read_operands(def: &OpDef, ins: Vec<u8>) -> (Vec<usize>, usize) {
    let mut ops: Vec<usize> = Vec::with_capacity(def.op_width.len());
    let mut offset = 0;

    for wd in &def.op_width {
        match wd {
            2 => {
                ops.push(BigEndian::read_u16(&ins[offset..offset + 2]) as usize);
                offset += 2;
            }
            1 => {
                ops.push(ins[offset] as usize);
                offset += 1;
            }
            0 => {}
            _ => panic!("unsupported operand width"),
        }
    }

    (ops, offset)
}

impl Display for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();

        let mut i = 0;

        while i < self.ins.len() {
            let op = self.ins[i];
            let opcode = u8_to_op(op);

            let def = get_def(&opcode);

            let (ops, sz) = read_operands(&def, self.ins[i + 1..].to_vec());
            res.push_str(&format!("{:04} {}\n", i, Self::fmt_ins(&def, &ops)));
            i += 1 + sz;
        }

        write!(f, "{res}")
    }
}

pub fn u8_to_op(o: u8) -> Opcode {
    unsafe { ::std::mem::transmute(o) }
}

impl Default for Instructions {
    fn default() -> Self {
        Self::new()
    }
}
impl Instructions {
    pub const fn new() -> Self {
        Self { ins: Vec::new() }
    }
    pub fn fmt_ins(def: &OpDef, ops: &Vec<usize>) -> String {
        if def.op_width.len() != ops.len() {
            return format!(
                "not enough operands for defination; W=>{} G=>{}",
                def.op_width.len(),
                ops.len()
            );
        }
        match def.op_width.len() {
            0 => def.name.to_string(),
            1 => format!("{} {}", def.name, ops[0]),
            2 => format!("{} {} {}", def.name, ops[0], ops[1]),
            _ => "ERR=> unsupported operand width".to_string(),
        }
    }

    pub fn add_ins(&mut self, i: Vec<u8>) {
        self.ins.extend_from_slice(&i)
    }

    pub fn read_uint16(insts: Vec<u8>, start: usize) -> u16 {
        let mut tc = Cursor::new(insts[start..].to_vec());

        if let Ok(v) = tc.read_u16::<BigEndian>() {
            return v;
        }

        0
    }

    pub fn read_u8(insts: &Vec<u8>) -> u8 {
        let mut tc = Cursor::new(insts);
        if let Ok(v) = tc.read_u8() {
            return v;
        }

        0
    }
}
