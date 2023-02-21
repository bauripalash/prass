use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub mod frame;
pub mod global;

use crate::{
    compiler::code::{self, Bytecode, Instructions},
    obj::{Closure, HashKey, HashPair, Object, ARRAY_OBJ, HASH_OBJ, NUMBER_OBJ, STRING_OBJ},
    token::NumberToken,
};

use self::frame::{Frame, FramePool};
use self::global::GlobalStack;

static STACK_SIZE: usize = 2048;

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

#[derive(Debug)]
pub struct Vm {
    constants: Vec<Rc<Object>>,
    stack: StackPool, //Vec<Object>,
    sp: usize,
    globals: GlobalStack, //Rc<RefCell<[Object]>>,
    frames: FramePool,
    frame_index: usize,
    last_popped: Rc<Object>,
}

//pub type Pframe = Rc<RefCell<Frame>>;

#[derive(Debug)]
pub struct StackPool {
    pub stack: Vec<Rc<Object>>,
    pub len: usize,
}

impl StackPool {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(STACK_SIZE),
            len: 0,
        }
    }

    pub fn push(&mut self, index: Option<usize>, obj: Rc<Object>) {
        if let Some(idx) = index {
            if idx >= self.len {
                self.stack.push(obj);
                self.len += 1;
            } else {
                self.stack[idx] = obj
            }
        } else {
            self.stack.push(obj);
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Rc<Object> {
        self.len -= 1;
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    pub fn get(&self, index: usize) -> &Rc<Object> {
        unsafe { self.stack.get_unchecked(index) }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Rc<Object> {
        unsafe { self.stack.get_unchecked_mut(index) }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

impl Vm {
    pub fn new(bc: Bytecode) -> Self {
        let main_cl = Rc::new(Closure::new(bc.instructions));
        let main_frame = Frame::new(main_cl, 0);
        let mut frames = FramePool::new();
        frames.frames = vec![Rc::new(RefCell::new(main_frame))];
        frames.len += 1;
        Self {
            constants: bc.constants,
            stack: StackPool::new(), //Vec::with_capacity(STACK_SIZE),
            globals: GlobalStack::new(),
            sp: 0,
            frames,
            frame_index: 1,
            last_popped: Rc::new(NULL),
        }
    }

    pub fn top_stack(&self) -> Rc<Object> {
        if self.sp == 0 {
            Rc::new(Object::Null)
        } else {
            //&self.stack[self.sp - 1]
            Rc::clone(self.stack.get(self.sp - 1))
        }
    }

    fn current_frame(&self) -> &Rc<RefCell<Frame>> {
        //&self.frames.frames[self.frame_index - 1]
        unsafe { self.frames.frames.get_unchecked(self.frame_index - 1) }
    }

    fn push_frame(&mut self, f: Frame) {
        if self.frame_index >= self.frames.len {
            self.frames.frames.push(Rc::new(RefCell::new(f)))
        } else {
            unsafe {
                (*self.frames.frames.get_unchecked(self.frame_index).as_ptr()) = f;
            }
        }

        self.frame_index += 1;
    }

    fn pop_frame(&mut self) -> Rc<RefCell<Frame>> {
        self.frame_index -= 1;

        self.frames.frames.pop().unwrap()
    }

    fn adv_ip(&mut self, by: usize) {
        unsafe {
            let ptr = self
                .frames
                .frames
                .get_unchecked(self.frame_index - 1)
                .as_ptr();
            (*ptr).ip += by as i64
        }
    }

    fn set_ip(&mut self, t: usize) {
        unsafe {
            let ptr = self
                .frames
                .frames
                .get_unchecked(self.frame_index - 1)
                .as_ptr();
            (*ptr).ip = t as i64;
        }
    }

    fn get_ip(&self) -> usize {
        unsafe {
            let ptr = self
                .frames
                .frames
                .get_unchecked(self.frame_index - 1)
                .as_ptr();
            (*ptr).ip as usize
        }
    }

    fn get_cur_frame_ins(&self) -> Rc<Instructions> {
        unsafe {
            let ptr = self
                .frames
                .frames
                .get_unchecked(self.frame_index - 1)
                .as_ptr();
            (*ptr).get_instructions()
        }
    }

    pub fn run(&mut self) {
        while self.current_frame().borrow().get_ip()
            < self.current_frame().borrow().get_ins_len() - 1
        {
            self.adv_ip(1);
            let ip = self.get_ip();
            let ins = self.get_cur_frame_ins();
            let op = code::u8_to_op(ins.ins[ip]);

            match op {
                code::Opcode::Const => {
                    let op_ins = &ins.ins;
                    let con_index = Instructions::read_uint16(op_ins, ip + 1) as usize;
                    let con_obj = self.constants[con_index].clone();
                    self.push(con_obj);

                    //println!("{con_index:?}");
                    //ip += 2;
                    self.adv_ip(2);
                }
                code::Opcode::Pop => {
                    self.last_popped = self.pop();
                }
                code::Opcode::Add
                | code::Opcode::Sub
                | code::Opcode::Mul
                | code::Opcode::Div
                | code::Opcode::Mod => self.exe_binary_op(op),

                code::Opcode::True => self.push(Rc::new(TRUE)),
                code::Opcode::False => self.push(Rc::new(FALSE)),
                code::Opcode::Equal | code::Opcode::NotEqual | code::Opcode::GT => {
                    self.exe_comparison(op)
                }
                code::Opcode::Bang => self.exe_bang_op(),
                code::Opcode::Minus => self.exe_pref_minux(),
                code::Opcode::Null => self.push(Rc::new(NULL)),
                code::Opcode::SetGlobal => {
                    let gi = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);
                    //self.globals[gi] = self.pop()
                    let pop_item = self.pop();
                    self.globals.push_value(gi, pop_item);
                }
                code::Opcode::GetGlobal => {
                    let gi = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    //self.push(&self.globals[gi].clone())
                    self.push(self.globals.get_value(gi))
                }
                code::Opcode::Jump => {
                    let pos = code::Instructions::read_uint16(&ins.ins, ip + 1);
                    //ip = (pos - 1) as usize
                    self.set_ip((pos - 1) as usize)
                }

                code::Opcode::JumpNotTruthy => {
                    let pos = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;

                    //ip += 2;
                    self.adv_ip(2);

                    let cond = self.pop();

                    if !self.is_obj_truthy(&cond) {
                        //ip = pos - 1;
                        self.set_ip(pos - 1)
                    }
                }
                code::Opcode::Array => {
                    let num_of_elms = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    let arr = self.build_arr(self.sp - num_of_elms, self.sp);
                    self.sp -= num_of_elms;
                    self.push(Rc::new(arr));
                }

                code::Opcode::Hash => {
                    let num_of_elms = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    let hash = self.build_hash(self.sp - num_of_elms, self.sp);
                    self.sp -= num_of_elms;

                    self.push(Rc::new(hash))
                }
                code::Opcode::Index => {
                    let index = self.pop();
                    let left = self.pop();
                    self.exe_index_expr(left, index)
                }
                code::Opcode::ReturnValue => {
                    let rvalue = self.pop();
                    let frm = self.pop_frame();
                    //self.pop();
                    self.sp = frm.borrow().get_bp() as usize - 1;
                    //frm.get_bp() as usize - 1;
                    //(*frm).borrow().get_bp() as usize - 1; //frm.as_ref().borrow().get_bp() as usize - 1; //frm.bp as usize - 1;
                    self.push(rvalue)
                }
                code::Opcode::Return => {
                    let frm = self.pop_frame();
                    self.sp = frm.borrow().get_bp() as usize - 1;
                    //(*frm).borrow().get_bp() as usize - 1; //frm.as_ref().borrow().get_bp() as usize -1; //frm.bp as usize - 1;
                    //self.pop();
                    self.push(Rc::new(NULL));
                }
                code::Opcode::SetLocal => {
                    let local_index = code::Instructions::read_u8(&ins.ins[ip + 1..]);
                    self.adv_ip(1);
                    let frm = self.current_frame().borrow().get_bp();
                    //self.current_frame().as_ref().borrow().get_bp(); //self.current_frame().bp;
                    //self.stack[(frm as usize) + (local_index as usize)] = self.pop()
                    //
                    let pop_item = self.pop();
                    self.stack
                        .push(Some(frm as usize + local_index as usize), pop_item)
                }
                code::Opcode::GetLocal => {
                    let local_index = code::Instructions::read_u8(&ins.ins[ip + 1..]) as usize;
                    self.adv_ip(1);
                    let frm_bp = self.current_frame().borrow().get_bp() as usize;
                    //self.current_frame().as_ref().borrow().get_bp() as usize; //self.current_frame().bp as usize;
                    //unsafe {
                    let stack_obj = Rc::clone(self.stack.get(frm_bp + local_index));

                    self.push(stack_obj);
                    //}

                    //self.stack[frm_bp + local_index].clone();
                }
                code::Opcode::Call => {
                    let num_args = code::Instructions::read_u8(&ins.ins[ip + 1..]);
                    self.adv_ip(1);
                    self.call_func(num_args as usize);
                }
                code::Opcode::Closure => {
                    let const_index = code::Instructions::read_uint16(&ins.ins, ip + 1);
                    let num_free = code::Instructions::read_u8(&ins.ins[ip + 3..]);

                    self.adv_ip(3);
                    self.push_closure(const_index as usize, num_free as usize);
                }
                code::Opcode::GetFree => {
                    let f_index = code::Instructions::read_u8(&ins.ins[ip + 1..]);
                    self.adv_ip(1);
                    //                    let curframe = self.current_frame();

                    let ccl = Rc::clone(&self.current_frame().borrow().cl);
                    //Rc::clone(&self.current_frame().cl);
                    //Rc::clone(&(*self.current_frame()).borrow().cl); //&curframe.as_ref().borrow().cl; //&self.current_frame().cl.clone();

                    self.push(Rc::clone(&ccl.frees[f_index as usize]))
                }

                code::Opcode::CurrentClosure => {
                    //                    let ccl = self.current_frame().cl.clone();
                    let ccl = Rc::clone(&self.current_frame().borrow().cl);
                    //Rc::clone(&self.current_frame().cl);
                    //&self.current_frame().as_ref().borrow().cl.clone();

                    self.push(Rc::new(Object::Closure(ccl)));
                }
                code::Opcode::Show => {
                    let num_items = code::Instructions::read_u8(&ins.ins[ip + 1..]) as usize;

                    let mut i = 0;

                    //                    let mut objs: Vec<Object> = Vec::new();
                    let mut result: Vec<String> = Vec::with_capacity(num_items);

                    while i < num_items {
                        result.push(self.pop().to_string());
                        //result.push(' ');
                        //objs.push(self.pop());
                        i += 1
                    }
                    result.reverse();
                    //let mut result = String::new();

                    println!("{}", result.join(" "));
                    self.adv_ip(2);
                }

                _ => {}
            }
            //ip += 1;
        }
    }

    fn push_closure(&mut self, index: usize, num_free: usize) {
        let obj = &self.constants[index];

        let Object::Compfunc(cf) = obj.as_ref() else{
            panic!("not fun");
        };

        let mut fr: Vec<Rc<Object>> = Vec::with_capacity(num_free); //vec![NULL.into(); num_free];
        let mut i = 0;
        while i < num_free {
            //fr[i] = self.stack[self.sp - num_free + i].clone().into();
            fr.push(Rc::clone(self.stack.get(self.sp - num_free + i)));
            i += 1;
        }

        self.sp -= num_free;

        let cls = Object::Closure(Rc::new(Closure {
            fun: cf.clone(),
            frees: fr,
        }));
        self.push(Rc::new(cls));
    }

    fn call_func(&mut self, num_args: usize) {
        let stack_object = Rc::clone(self.stack.get(self.sp - 1 - num_args)); //&self.stack[self.sp - 1 - num_args].clone();
                                                                              //println!("STACK-OBJ{:?}" , stack_object);
                                                                              //let Object::Closure(cf) = stack_object else{
                                                                              //                panic!("stack object is not compiled function")
                                                                              //            } ;
        let Object::Closure(cf) = &*stack_object else{
                println!("not closure");
                if let Object::Closure(lcf) = self.last_pop(){

                    self.call_closure(lcf, num_args);
                    return;

                };

                    panic!("not closure -> panic");

        };

        self.call_closure(cf.clone(), num_args);
    }

    fn call_closure(&mut self, cal: Rc<Closure>, num_args: usize) {
        if cal.fun.num_params != num_args {
            panic!(
                "arg number and params number is not same| W=>{} G={}",
                cal.fun.num_params, num_args
            );
        }

        let frame = Frame::new(cal.clone(), (self.sp - num_args) as i64);
        let fbp = frame.bp as usize;
        self.push_frame(frame);

        self.sp = fbp + cal.fun.num_locals;
    }

    fn exe_index_expr(&mut self, left: Rc<Object>, index: Rc<Object>) {
        if left.get_type() == ARRAY_OBJ && index.get_type() == NUMBER_OBJ {
            self.exe_arr_index(left, index)
        } else if left.get_type() == HASH_OBJ {
            self.exe_hash_index(left, index)
        } else {
            panic!("index operator not supported -> {}", index.get_type())
        }
    }

    fn exe_arr_index(&mut self, arr: Rc<Object>, index: Rc<Object>) {
        let Object::Array { token : _ , value } = &*arr else { panic!("not array") };
        let id: Option<i64> = if let Object::Number { token: _, value } = &*index {
            Some(value.get_as_i64())
        } else {
            None
        };

        let max = (value.len() - 1) as i64;

        if id.unwrap() < 0 || id.unwrap() > max {
            self.push(Rc::new(NULL))
        } else {
            self.push(value[id.unwrap() as usize].clone())
        }
    }

    fn exe_hash_index(&mut self, hash: Rc<Object>, index: Rc<Object>) {
        let Object::Hash { token : _, pairs } = &*hash else{ panic!("not hash") };
        if !index.hashable() {
            panic!("index key is not hashable")
        }
        let hk = HashKey {
            key: index.get_hash(),
        };
        //println!("{:?}" , pairs);
        if let Some(v) = pairs.get(&hk) {
            self.push(v.value.clone())
        } else {
            self.push(Rc::new(NULL))
        }
    }

    fn build_hash(&mut self, start: usize, end: usize) -> Object {
        let mut hp: HashMap<Rc<HashKey>, Rc<HashPair>> = HashMap::new();

        let mut i = start;

        while i < end {
            let k: Rc<Object>;
            let v: Rc<Object>;
            //unsafe {
            //    k = *self.stack.get(i); //Rc::new(self.stack.get_unchecked(i).clone());
            //}
            k = Rc::clone(self.stack.get(i));

            //let v =
            //unsafe {
            //    v = *self.stack.get(i+1); //Rc::new(self.stack.get_unchecked(i + 1).clone());
            //}
            v = Rc::clone(self.stack.get(i + 1));
            if !k.hashable() {
                panic!("key is not hashable")
            }

            let hk = Rc::new(HashKey { key: k.get_hash() });
            hp.insert(hk, Rc::new(HashPair { key: k, value: v }));
            i += 2;
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
            elms[i - start] = Rc::clone(self.stack.get(i)); //Rc::new(self.stack[i].clone());
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

        let Object::Number { token : _, value } = &*op else {
            panic!("not a number")
        };

        self.push(Rc::new(Object::Number {
            token: None,
            value: value.make_neg(),
        }))
    }

    fn exe_bang_op(&mut self) {
        let o = self.pop();

        match *o {
            Object::Bool { token: _, value } => {
                if value {
                    self.push(Rc::new(FALSE))
                } else {
                    self.push(Rc::new(TRUE))
                }
            }
            Object::Null => self.push(Rc::new(TRUE)),
            _ => self.push(Rc::new(FALSE)),
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
            code::Opcode::Equal => self.push(Rc::new(bool_native_to_obj(right == left))),
            code::Opcode::NotEqual => self.push(Rc::new(bool_native_to_obj(left != right))),
            _ => {
                panic!("unknonwn operator -> {op:?}")
            }
        }
    }

    fn exe_comparison_number(&mut self, op: code::Opcode, left: Rc<Object>, right: Rc<Object>) {
        let lval: Option<NumberToken> = if let Object::Number { token: _, value } = &*left {
            Some(value.clone())
        } else {
            None
        };

        let rval: Option<NumberToken> = if let Object::Number { token: _, value } = &*right {
            Some(value.clone())
        } else {
            None
        };

        match op {
            code::Opcode::Equal => self.push(Rc::new(bool_native_to_obj(lval == rval))),
            code::Opcode::GT => self.push(Rc::new(bool_native_to_obj(lval > rval))),
            code::Opcode::NotEqual => self.push(Rc::new(bool_native_to_obj(lval != rval))),

            _ => panic!("unknown comparison"),
        }
    }

    fn exe_binary_op(&mut self, op: code::Opcode) {
        let right = self.pop();
        let left = self.pop();
        if right.get_type() == NUMBER_OBJ && left.get_type() == NUMBER_OBJ {
            self.exe_binary_op_number(op, left, right)
        } else if right.get_type() == STRING_OBJ && left.get_type() == STRING_OBJ {
            if op != code::Opcode::Add {
                panic!("only '+' is supported for strings")
            }

            let Object::String { token : _, value : lval } = &*left else{
                panic!("left object is not string")
            };

            let Object::String { token : _, value : rval } = &*right else{
                panic!("left object is not string")
            };

            self.push(Rc::new(Object::String {
                token: None,
                value: format!("{lval}{rval}"),
            }));

            //            self.exe_binary_op_str(op, left, right)
        }
    }

    fn exe_binary_op_number(&mut self, op: code::Opcode, left: Rc<Object>, right: Rc<Object>) {
        let Object::Number { token : _, value } = &*left else {
            panic!("not a number")
        };
        let lval = value.clone();

        let Object::Number { token : _, value } = &*right else {
            panic!("rval is not a number")
        };

        let rval = value.clone();
        let value: NumberToken;

        match op {
            code::Opcode::Add => value = lval + rval,
            code::Opcode::Sub => value = lval - rval,
            code::Opcode::Mul => value = lval * rval,
            code::Opcode::Div => value = lval / rval,
            code::Opcode::Mod => value = lval % rval,
            _ => {
                panic!("unknown {op:?} operator for numbers")
            }
        }

        self.push(Rc::new(Object::Number { token: None, value }));
    }

    fn push(&mut self, obj: Rc<Object>) {
        if self.sp >= STACK_SIZE {
            panic!("stack overflow");
        }
        if self.sp >= self.stack.len {
            //self.stack.push(obj.to_owned())
            self.stack.push(None, obj)
        } else {
            //self.stack[self.sp] = obj.to_owned()
            self.stack.push(Some(self.sp), obj)
        }
        //self.stack[self.sp] = obj.clone();
        //self.stack.push(obj.to_owned());
        self.sp += 1;
    }

    fn pop(&mut self) -> Rc<Object> {
        let ip = self.sp - 1;
        //let obj = &self.stack[ip];
        //
        if self.sp == self.stack.len() {
            self.sp -= 1;
            return self.stack.pop();
        }

        //let obj = self.stack.pop().unwrap();
        let obj = self.stack.get(ip); //&self.stack[ip];
        self.sp -= 1;
        obj.clone()
    }

    pub fn last_pop(&self) -> Object {
        self.last_popped.as_ref().to_owned()
        //println!("LAST-POP->{:?}->{:?}" , self.stack , self.last_popped);
        //if let Some(lp) = self.stack.get(self.sp){
        //    return lp.to_owned()
        //}else{
        //    return self.last_popped.as_ref().to_owned()
        //}
        //self.stack[self.sp].clone()
    }
}
