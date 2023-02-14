use std::{collections::BTreeMap, rc::Rc};

pub mod frame;

use crate::{
    compiler::code::{self, Bytecode, Instructions},
    obj::{CompFunc, HashKey, HashPair, Object, ARRAY_OBJ, HASH_OBJ, NUMBER_OBJ, STRING_OBJ},
    token::NumberToken,
};

use self::frame::Frame;

const STACK_SIZE: usize = 2048;
const GLOBALS_SIZE: usize = 1024; //Change
static FRAMES_SIZE: usize = 1024;

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
    //pub instructions: code::Instructions,
    stack: Vec<Object>,
    sp: usize,
    globals: Vec<Object>,
    frames: Vec<Frame>,
    frame_index: usize,
}

impl Vm {
    pub fn new(bc: Bytecode) -> Self {
        let main_func = CompFunc::new(bc.instructions);
        let main_frame = Frame::new(main_func);
        let mut frames: Vec<Frame> = vec![Frame::default(); FRAMES_SIZE];
        frames[0] = main_frame;
        Self {
            constants: bc.constants,
            stack: vec![Object::Null; STACK_SIZE],
            globals: vec![Object::Null; GLOBALS_SIZE],
            sp: 0,
            frames,
            frame_index: 1,
        }
    }

    pub fn top_stack(&self) -> &Object {
        if self.sp == 0 {
            &Object::Null
        } else {
            &self.stack[self.sp - 1]
        }
    }

    fn current_frame(&self) -> &Frame {
        &self.frames[self.frame_index - 1]
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        &mut self.frames[self.frame_index - 1]
    }

    fn push_frame(&mut self, f: Frame) {
        self.frames[self.frame_index] = f;
        self.frame_index += 1;
    }

    fn pop_frame(&mut self) -> Frame {
        self.frame_index -= 1;
        self.frames[self.frame_index].clone()
    }

    fn adv_ip(&mut self, by: usize) {
        self.current_frame_mut().ip += by as i64
    }

    fn set_ip(&mut self, t: usize) {
        self.current_frame_mut().ip = t as i64
    }

    pub fn run(&mut self) {
        let mut ip: usize;
        let mut ins: Instructions;
        let mut op: code::Opcode;
        while self.current_frame().ip
            < (self.current_frame().get_instructions().ins.len() as i64) - 1
        {
            //self.current_frame_mut().ip += 1;
            self.adv_ip(1);
            ip = self.current_frame().ip as usize;
            ins = self.current_frame().get_instructions();
            op = code::u8_to_op(ins.ins[ip]);
            //println!("{:?}", op);

            match op {
                code::Opcode::OpConst => {
                    let op_ins = ins.ins;
                    let con_index = Instructions::read_uint16(op_ins.to_vec(), ip + 1) as usize;
                    let con_obj = &self.constants[con_index].clone();
                    self.push(con_obj);

                    //println!("{con_index:?}");
                    //ip += 2;
                    self.adv_ip(2);
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
                    let gi = code::Instructions::read_uint16(ins.ins.to_vec(), ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);
                    self.globals[gi] = self.pop()
                }
                code::Opcode::OpGetGlobal => {
                    let gi = code::Instructions::read_uint16(ins.ins.to_vec(), ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    self.push(&self.globals[gi].clone())
                }
                code::Opcode::OpJump => {
                    let pos = code::Instructions::read_uint16(ins.ins.to_vec(), ip + 1);
                    //ip = (pos - 1) as usize
                    self.set_ip((pos - 1) as usize)
                }

                code::Opcode::OpJumpNotTruthy => {
                    let pos = code::Instructions::read_uint16(ins.ins.to_vec(), ip + 1) as usize;

                    //ip += 2;
                    self.adv_ip(2);

                    let cond = self.pop();

                    if !self.is_obj_truthy(&cond) {
                        //ip = pos - 1;
                        self.set_ip(pos - 1)
                    }
                }
                code::Opcode::OpArray => {
                    let num_of_elms =
                        code::Instructions::read_uint16(ins.ins.to_vec(), ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    let arr = self.build_arr(self.sp - num_of_elms, self.sp);
                    self.sp -= num_of_elms;
                    self.push(&arr);
                }

                code::Opcode::OpHash => {
                    let num_of_elms =
                        code::Instructions::read_uint16(ins.ins.to_vec(), ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    let hash = self.build_hash(self.sp - num_of_elms, self.sp);
                    self.sp -= num_of_elms;

                    self.push(&hash)
                }
                code::Opcode::OpIndex => {
                    let index = self.pop();
                    let left = self.pop();
                    self.exe_index_expr(left, index)
                }
                code::Opcode::OpCall => {
                    let stack_object = &self.stack[self.sp - 1];
                    let Object::Compfunc(cf) = stack_object else{
                        panic!("stack object is not compiled function")
                    } ;
                    let frm = Frame::new(cf.clone());
                    self.push_frame(frm)
                }
                code::Opcode::OpReturnValue => {
                    let rvalue = self.pop();
                    self.pop_frame();
                    self.pop();
                    self.push(&rvalue)
                }
                code::Opcode::OpReturn => {
                    self.pop_frame();
                    self.pop();
                    self.push(&NULL);
                }

                _ => {}
            }
            //ip += 1;
        }
    }

    fn exe_index_expr(&mut self, left: Object, index: Object) {
        if left.get_type() == ARRAY_OBJ && index.get_type() == NUMBER_OBJ {
            self.exe_arr_index(left, index)
        } else if left.get_type() == HASH_OBJ {
            self.exe_hash_index(left, index)
        } else {
            panic!("index operator not supported -> {}", index.get_type())
        }
    }

    fn exe_arr_index(&mut self, arr: Object, index: Object) {
        let Object::Array { token : _ , value } = arr else { panic!("not array") };
        let id: Option<i64> = if let Object::Number { token: _, value } = index {
            Some(value.get_as_i64())
        } else {
            None
        };

        let max = (value.len() - 1) as i64;

        if id.unwrap() < 0 || id.unwrap() > max {
            self.push(&NULL)
        } else {
            self.push(&value[id.unwrap() as usize])
        }
    }

    fn exe_hash_index(&mut self, hash: Object, index: Object) {
        let Object::Hash { token : _, pairs } = hash else{ panic!("not hash") };
        if !index.hashable() {
            panic!("index key is not hashable")
        }
        let hk = HashKey {
            key: index.get_hash(),
        };
        //println!("{:?}" , pairs);
        if let Some(v) = pairs.get(&hk) {
            self.push(&v.value.clone())
        } else {
            self.push(&NULL)
        }
    }

    fn build_hash(&mut self, start: usize, end: usize) -> Object {
        let mut hp: BTreeMap<Rc<HashKey>, Rc<HashPair>> = BTreeMap::new();

        let mut i = start;

        while i < end {
            let k = Rc::new(self.stack[i].clone());
            let v = Rc::new(self.stack[i + 1].clone());
            if !k.hashable() {
                panic!("key is not hashable")
            }

            let hk = Rc::new(HashKey { key: k.get_hash() });
            hp.insert(hk, Rc::new(HashPair { key: k, value: v }));
            i += 1;
        }

        Object::Hash {
            token: None,
            pairs: hp,
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

        let Object::Number { token : _, value } = op else {
            panic!("not a number")
        };

        self.push(&Object::Number {
            token: None,
            value: value.make_neg(),
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
