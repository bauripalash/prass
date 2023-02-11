use crate::{
    compiler::code::{self, Bytecode, Instructions},
    obj::Object,
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
                code::Opcode::OpAdd => {
                    let right = self.pop();
                    let left = self.pop();
                    let lval = match left {
                        Object::Number { token: _, value } => value.get_as_i64(),

                        _ => 0_i64,
                    };

                    let rval = match right {
                        Object::Number { token: _, value } => value.get_as_i64(),

                        _ => 0_i64,
                    };

                    let result = lval + rval;

                    self.push(&Object::Number {
                        token: Token::dummy().into(),
                        value: NumberToken::from(result),
                    })
                }

                _ => {}
            }
            ip += 1;
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
        let obj = &self.stack[self.sp - 1];
        self.sp -= 1;
        obj.clone()
    }
}
