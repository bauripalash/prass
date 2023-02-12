use std::rc::Rc;

use crate::{
    compiler::code::{self, Bytecode, Instructions},
    obj::{Object, NUMBER_OBJ, STRING_OBJ},
    token::NumberToken,
};

const STACK_SIZE: usize = 2048;
const GLOBALS_SIZE: usize = 1024; //Change

const TRUE: Object = Object::Bool {
    token: None,
    value: true,
};
const FALSE: Object = Object::Bool {
    token: None,
    value: false,
};
const NULL: Object = Object::Null;

const fn bool_native_to_obj(b: bool) -> Object {
    if b {
        TRUE
    } else {
        FALSE
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vm {
    constants: Vec<Object>,
    pub instructions: code::Instructions,
    stack: Vec<Object>,
    sp: usize,
    globals: Vec<Object>,
}

impl Vm {
    pub fn new(bc: Bytecode) -> Self {
        Self {
            constants: bc.constants,
            instructions: bc.instructions,
            stack: vec![Object::Null; STACK_SIZE],
            globals: vec![Object::Null; GLOBALS_SIZE],
            sp: 0,
        }
    }

    pub fn top_stack(&self) -> &Object {
        if self.sp == 0 {
            &Object::Null
        } else {
            &self.stack[self.sp - 1]
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
                    self.push(con_obj);

                    //println!("{con_index:?}");
                    ip += 2;
                }
                code::Opcode::OpPop => {
                    self.pop();
                }
                code::Opcode::OpAdd
                | code::Opcode::OpSub
                | code::Opcode::OpMul
                | code::Opcode::OpDiv
                | code::Opcode::OpMod => self.exe_binary_op(op),

                code::Opcode::OpTrue => self.push(&TRUE),
                code::Opcode::OpFalse => self.push(&FALSE),
                code::Opcode::OpEqual | code::Opcode::OpNotEqual | code::Opcode::OpGT => {
                    self.exe_comparison(op)
                }
                code::Opcode::OpBang => self.exe_bang_op(),
                code::Opcode::OpMinus => self.exe_pref_minux(),
                code::Opcode::OpNull => self.push(&NULL),
                code::Opcode::OpSetGlobal => {
                    let gi = code::Instructions::read_uint16(self.instructions.ins.to_vec(), ip + 1)
                        as usize;
                    ip += 2;
                    self.globals[gi] = self.pop()
                }
                code::Opcode::OpGetGlobal => {
                    let gi = code::Instructions::read_uint16(self.instructions.ins.to_vec(), ip + 1)
                        as usize;
                    ip += 2;

                    self.push(&self.globals[gi].clone())
                }
                code::Opcode::OpJump => {
                    let pos =
                        code::Instructions::read_uint16(self.instructions.ins.to_vec(), ip + 1);
                    ip = (pos - 1) as usize
                }

                code::Opcode::OpJumpNotTruthy => {
                    let pos =
                        code::Instructions::read_uint16(self.instructions.ins.to_vec(), ip + 1)
                            as usize;

                    ip += 2;

                    let cond = self.pop();

                    if !self.is_obj_truthy(&cond) {
                        ip = pos - 1;
                    }
                }
                code::Opcode::OpArray => {
                    let num_of_elms =
                        code::Instructions::read_uint16(self.instructions.ins.to_vec(), ip + 1)
                            as usize;
                    ip += 2;

                    let arr = self.build_arr(self.sp - num_of_elms, self.sp);
                    self.sp -= num_of_elms;
                    self.push(&arr);
                }

                _ => {}
            }
            ip += 1;
        }
    }

    fn build_arr(&mut self, start: usize, end: usize) -> Object {
        let mut elms: Vec<Rc<Object>> = {
            let data = Rc::new(NULL);
            vec![data; end - start]
        };
        let mut i = start;

        while i < end {
            elms[i - start] = Rc::new(self.stack[i].clone());
            i += 1;
        }

        Object::Array {
            token: None,
            value: elms,
        }
    }
    const fn is_obj_truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::Bool { token: _, value } => *value,
            Object::Null => false,
            _ => true,
        }
    }

    fn exe_pref_minux(&mut self) {
        let op = self.pop();

        if op.get_type() != NUMBER_OBJ {
            panic!("negetion can only be applied on numbers -> {op:?}")
        }

        let val: Option<NumberToken> = match op {
            Object::Number { token: _, value } => Some(value),
            _ => None,
        };

        self.push(&Object::Number {
            token: None,
            value: val.unwrap().make_neg(),
        })
    }

    fn exe_bang_op(&mut self) {
        let o = self.pop();

        match o {
            Object::Bool { token: _, value } => {
                if value {
                    self.push(&FALSE)
                } else {
                    self.push(&TRUE)
                }
            }
            Object::Null => self.push(&TRUE),
            _ => self.push(&FALSE),
        };
    }

    fn exe_comparison(&mut self, op: code::Opcode) {
        let right = self.pop();
        let left = self.pop();
        if left.get_type() == NUMBER_OBJ && right.get_type() == NUMBER_OBJ {
            self.exe_comparison_number(op, left, right);
            return;
        }

        match op {
            code::Opcode::OpEqual => self.push(&bool_native_to_obj(right == left)),
            code::Opcode::OpNotEqual => self.push(&bool_native_to_obj(left != right)),
            _ => {
                panic!("unknonwn operator -> {op:?}")
            }
        }
    }

    fn exe_comparison_number(&mut self, op: code::Opcode, left: Object, right: Object) {
        let lval: Option<NumberToken> = if let Object::Number { token: _, value } = left {
            Some(value)
        } else {
            None
        };

        let rval: Option<NumberToken> = if let Object::Number { token: _, value } = right {
            Some(value)
        } else {
            None
        };

        match op {
            code::Opcode::OpEqual => self.push(&bool_native_to_obj(lval == rval)),
            code::Opcode::OpGT => self.push(&bool_native_to_obj(lval > rval)),
            code::Opcode::OpNotEqual => self.push(&bool_native_to_obj(lval != rval)),

            _ => panic!("unknown comparison"),
        }
    }

    fn exe_binary_op(&mut self, op: code::Opcode) {
        let right = self.pop();
        let left = self.pop();
        if right.get_type() == NUMBER_OBJ && left.get_type() == NUMBER_OBJ {
            self.exe_binary_op_number(op, left, right)
        } else if right.get_type() == STRING_OBJ && left.get_type() == STRING_OBJ {
            self.exe_binary_op_str(op, left, right)
        }
    }

    fn exe_binary_op_str(&mut self, op: code::Opcode, left: Object, right: Object) {
        if op != code::Opcode::OpAdd {
            panic!("unknown string operator : {op:?}")
        }
        let lval: Option<String> = if let Object::String { token: _, value } = left {
            Some(value)
        } else {
            None
        };
        let rval: Option<String> = if let Object::String { token: _, value } = right {
            Some(value)
        } else {
            None
        };

        self.push(&Object::String {
            token: None,
            value: lval.unwrap() + &rval.unwrap(),
        })
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
                code::Opcode::OpMod => lfv % rfv,
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
                code::Opcode::OpMod => lfv % rfv,
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
