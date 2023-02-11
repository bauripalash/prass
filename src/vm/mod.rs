use crate::{
    compiler::code::{self, Bytecode, Instructions},
    obj::{Object, NUMBER_OBJ},
    token::{NumberToken, Token, TokenType},
};

const STACK_SIZE: usize = 2048;

#[derive(Debug, Clone, PartialEq)]
pub struct Vm {
    constants: Vec<Object>,
    pub instructions: code::Instructions,
    stack: Vec<Object>,
    sp: usize,
}

impl Vm {
    pub fn new(bc: Bytecode) -> Self {
        Self {
            constants: bc.constants,
            instructions: bc.instructions,
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
        }
    }

    pub fn top_stack(&self) -> &Object {
        if self.sp == 0 {
            return &Object::Null;
        } else {
            return &self.stack[self.sp - 1];
        }
    }

    pub fn run(&mut self) {
        let mut ip = 0;
        while ip < self.instructions.ins.len() {
            let op = code::u8_to_op(self.instructions.ins[ip]);
            //println!("{:?}", op);

            match op {
                code::Opcode::OpConst => {
                    let op_ins = &self.instructions.ins;
                    let con_index = Instructions::read_uint16(op_ins.to_vec(), ip + 1) as usize;
                    let con_obj = &self.constants[con_index].clone();
                    self.push(&con_obj);

                    //println!("{con_index:?}");
                    ip += 2;
                }
                code::Opcode::OpPop => {
                    self.pop();
                }
                code::Opcode::OpAdd
                | code::Opcode::OpSub
                | code::Opcode::OpMul
                | code::Opcode::OpDiv => self.exe_binary_op(op),

                _ => {}
            }
            ip += 1;
        }
    }

    fn exe_binary_op(&mut self, op: code::Opcode) {
        let right = self.pop();
        let left = self.pop();
        if right.get_type() == NUMBER_OBJ && left.get_type() == NUMBER_OBJ {
            self.exe_binary_op_number(op, left, right)
        }
    }

    fn exe_binary_op_number(&mut self, op: code::Opcode, left: Object, right: Object) {
        let mut is_float = false;
        let lval: Option<NumberToken> = if let Object::Number { token: _, value } = left {
            is_float = value.get_type();
            Some(value)
        } else {
            None
        };
        let rval: Option<NumberToken> = if let Object::Number { token: _, value } = right {
            is_float = value.get_type();
            Some(value)
        } else {
            None
        };

        if is_float {
            let lfv = lval.unwrap().get_as_f64();
            let rfv = rval.unwrap().get_as_f64();
            let value = NumberToken::from(match op {
                code::Opcode::OpAdd => lfv + rfv,
                code::Opcode::OpSub => lfv - rfv,
                code::Opcode::OpMul => lfv * rfv,
                code::Opcode::OpDiv => lfv / rfv,
                _ => 0.0,
            });
            self.push(&Object::Number { token: None, value });
        } else {
            let lfv = lval.unwrap().get_as_i64();
            let rfv = rval.unwrap().get_as_i64();
            let value = NumberToken::from(match op {
                code::Opcode::OpAdd => lfv + rfv,
                code::Opcode::OpSub => lfv - rfv,
                code::Opcode::OpMul => lfv * rfv,
                code::Opcode::OpDiv => lfv / rfv,
                _ => 0,
            });
            self.push(&Object::Number { token: None, value });
        }
    }

    fn push(&mut self, obj: &Object) {
        if self.sp >= STACK_SIZE {
            panic!("stack overflow");
        }

        self.stack[self.sp] = obj.clone();
        self.sp += 1;
    }

    fn pop(&mut self) -> Object {
        let ip = self.sp - 1;
        let obj = &self.stack[ip];
        self.sp -= 1;
        obj.clone()
    }

    pub fn last_pop(&self) -> Object {
        self.stack[self.sp].clone()
    }
}
