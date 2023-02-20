use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

pub mod frame;

use crate::{
    compiler::code::{self, Bytecode, Instructions},
    obj::{Closure, HashKey, HashPair, Object, ARRAY_OBJ, HASH_OBJ, NUMBER_OBJ, STRING_OBJ},
    token::NumberToken,
};

use self::frame::Frame;

static STACK_SIZE: usize = 2048;
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

#[derive(Debug)]
pub struct Vm {
    constants: Vec<Rc<Object>>,
    stack: Vec<Object>,
    sp: usize,
    globals: GlobalStack, //Rc<RefCell<[Object]>>,
    frames: FramePool,
    frame_index: usize,
    last_popped: Rc<Object>,
}

//pub type Pframe = Rc<RefCell<Frame>>;

#[derive(Debug)]
struct StackPool {
    pub stack: Vec<Rc<RefCell<Object>>>,
    pub len: usize,
}

impl StackPool {
    pub const fn new() -> Self {
        Self {
            stack: Vec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, index: usize, obj: Object) {
        if index >= self.len {
            self.stack.push(Rc::new(RefCell::new(obj)));
            self.len += 1
        } else {
            unsafe {
                let ptr = self.stack[index].as_ptr();
                *ptr = obj;
            }
        }
    }

    pub fn pop(&mut self) -> Rc<RefCell<Object>> {
        self.len -= 1;
        self.stack.pop().unwrap()
    }

    pub fn get(&self, index: usize) -> &Rc<RefCell<Object>> {
        &self.stack[index]
    }

    pub fn get_mut(&self, index: usize) -> RefMut<Object> {
        //self.stack[index].as_ptr();
        //ptr
        self.stack[index].borrow_mut()
    }
}

#[derive(Debug)]
struct FramePool {
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

    pub fn push_frame(&mut self, index: usize, frame: Frame) {
        if index >= self.len {
            //self.frames.push(Rc::new(RefCell::new(frame)));
            //self.frames.push(Box::new(frame));
            self.frames.push(Rc::new(RefCell::new(frame)));
            self.len += 1;
        } else {
            //self.frames[index] = Rc::new(RefCell::new(frame)) //Pframe::new_from_frame(frame);
            //self.frames[index] = Rc::new(RefCell::new(frame)) // Arc::new(frame)
            unsafe {
                let x = self.frames[index].as_ptr();
                *x = frame;
            }
        }
    }

    pub fn pop_frame(&mut self) -> Rc<RefCell<Frame>> {
        self.len -= 1;
        self.frames.pop().unwrap()
        //let mut r = self.frames.pop().unwrap();
        //Arc::get_mut(&mut r).unwrap()
        //Arc::get_mut(&mut self.frames[index]).unwrap()
    }

    pub fn get_frame_mut(&mut self, index: usize) -> RefMut<Frame> {
        //Rc::clone(&self.frames[index])
        //Arc::get_mut(&mut self.frames[index]).unwrap()
        self.frames[index].borrow_mut()
    }

    pub fn get_frame(&self, index: usize) -> &Rc<RefCell<Frame>> {
        //Arc::clone(&self.frames[index])
        &self.frames[index]
    }

    pub fn adv_ip(&mut self, index: usize, by: i64) {
        //self.frames[index].adv_ip(by)
        //Arc::get_mut(&mut self.frames[index]).unwrap().adv_ip(by)
        //let x = self.frames[index].as_ptr() as *const Frame;
        //unsafe {
        //    x
        //}
        //let x = &self.frames[index];
        //(*x.borrow_mut()).adv_ip(by)
        //x.adv_ip(by)
        //(*self.frames.get(index).unwrap()).borrow_mut().adv_ip(by)
        unsafe {
            let x = &self.frames[index];
            let ptr = x.as_ptr();
            (*ptr).ip += by;
        }
    }

    pub fn set_ip(&mut self, index: usize, by: i64) {
        //self.frames[index].set_ip(by);
        //Arc::get_mut(&mut self.frames[index]).unwrap().set_ip(by)
        //(*self.frames[index].borrow_mut()).set_ip(by)
        unsafe {
            let ptr = self.frames[index].as_ptr();
            (*ptr).ip = by;
        }
    }

    pub fn get_ip(&self, index: usize) -> i64 {
        //self.frames[index].get_ip()
        //self.frames[index].borrow().get_ip()
        let ptr = self.frames[index].as_ptr();
        unsafe { (*ptr).ip }
    }

    pub fn get_ins(&self, index: usize) -> Rc<Instructions> {
        //self.frames.borrow()[index].as_ref().borrow().get_instructions()
        //self.frames[index].get_instructions()
        //self.frames[index].borrow().get_instructions()
        let ptr = self.frames[index].as_ptr();
        unsafe { (*ptr).get_instructions() }
    }
}

#[derive(Debug)]
struct GlobalStack {
    pub globals: RefCell<Vec<Rc<Object>>>,
    pub len: usize,
}

impl GlobalStack {
    pub fn new() -> Self {
        Self {
            globals: RefCell::new(Vec::with_capacity(GLOBALS_SIZE)),
            len: 0,
        }
    }
    pub fn push_value(&mut self, index: usize, obj: Rc<Object>) {
        if index >= self.len {
            self.globals.borrow_mut().push(obj);
            self.len += 1;
        } else {
            //self.globals.borrow_mut()[index] = obj;
            let ptr = self.globals.as_ptr();
            unsafe {
                (*ptr)[index] = obj;
            }
        }
    }

    pub fn get_value(&self, index: usize) -> Rc<Object> {
        if index >= self.len || index > GLOBALS_SIZE {
            Rc::new(Object::Null)
        } else {
            Rc::clone(&self.globals.borrow_mut()[index])
        }
    }
}

impl Vm {
    pub fn new(bc: Bytecode) -> Self {
        let main_cl = Rc::new(Closure::new(bc.instructions));
        let main_frame = Frame::new(main_cl, 0);
        //let mut frames: Vec<Frame> = vec![Frame::default(); FRAMES_SIZE];
        //frames[0] = main_frame;
        //let gl = Rc::new(RefCell::new([NULL;GLOBALS_SIZE]));
        let mut frames = FramePool::new();
        frames.push_frame(0, main_frame);
        Self {
            constants: bc.constants,
            stack: Vec::with_capacity(STACK_SIZE), //vec![Object::Null; STACK_SIZE],
            globals: GlobalStack::new(),           //vec![Object::Null; GLOBALS_SIZE],
            sp: 0,
            frames,
            frame_index: 1,
            last_popped: Rc::new(NULL),
        }
    }

    pub fn top_stack(&self) -> &Object {
        if self.sp == 0 {
            &Object::Null
        } else {
            &self.stack[self.sp - 1]
        }
    }

    fn current_frame(&self) -> &Rc<RefCell<Frame>> {
        //println!("->{:?}" , self.frames.get_frame(self.frame_index-1));
        //self.frames.get_frame(self.frame_index - 1)
        //&self.frames[self.frame_index - 1]
        //self.frames.get_frame(self.frame_index - 1)
        self.frames.get_frame(self.frame_index - 1)
    }

    //fn current_frame_mut(&mut self) -> Rc<RefCell<Frame>> {
    //self.frames.get_frame_mut(self.frame_index - 1)
    //    self.frames.get_frame(self.frame_index - 1)
    //}

    //fn current_frame_mut(&mut self) -> &mut Frame {
    //    &mut self.frames[self.frame_index - 1]
    //}

    fn push_frame(&mut self, f: Frame) {
        self.frames.push_frame(self.frame_index, f);
        //self.frames[self.frame_index] = f;

        self.frame_index += 1;
    }

    fn pop_frame(&mut self) -> Rc<RefCell<Frame>> {
        self.frame_index -= 1;
        //self.frames[self.frame_index].clone()
        //self.frames.get_frame(self.frame_index)
        //self.frames.get_frame(self.frame_index)
        self.frames.pop_frame()
    }

    fn adv_ip(&mut self, by: usize) {
        //self.current_frame_mut().ip += by as i64
        // self.frames[self.frame_index - 1].ip += by as i64
        //self.current_frame().borrow_mut()
        self.frames.adv_ip(self.frame_index - 1, by as i64)
    }

    fn set_ip(&mut self, t: usize) {
        //self.current_frame_mut().ip = t as i64
        //self.frames[self.frame_index - 1].ip = t as i64
        self.frames.set_ip(self.frame_index - 1, t as i64)
    }

    fn get_ip(&self) -> usize {
        //self.frames[self.frame_index - 1].ip
        self.frames.get_ip(self.frame_index - 1) as usize
    }

    fn get_cur_frame_ins(&self) -> Rc<Instructions> {
        self.frames.get_ins(self.frame_index - 1)
    }

    //fn get_ins_len(&self) -> i64 {
    //    self.current_frame().borrow().get_ins_len()
    //}

    pub fn run(&mut self) {
        //let mut ip: usize;
        //let mut ins: Rc<Instructions>;
        //let mut op: code::Opcode;
        //        let curframe = self.current_frame();
        //while self.get_ip() < (self.get_ins_len() - 1) as usize
        //< (self.current_frame().get_instructions().ins.len() as i64) - 1
        //while (*self.current_frame()).borrow().get_ip()
        //    < (*self.current_frame()).borrow().get_ins_len() - 1
        while self.current_frame().borrow().get_ip()
            < self.current_frame().borrow().get_ins_len() - 1
        {
            //self.current_frame().get_ins_len() - 1 {
            //while self.loop_cur_frame() {
            //self.current_frame_mut().ip += 1;
            self.adv_ip(1);
            let ip = self.get_ip();
            let ins = self.get_cur_frame_ins(); //self.current_frame().get_instructions();
            let op = code::u8_to_op(ins.ins[ip]);

            //println!("{:?}", op);

            match op {
                code::Opcode::Const => {
                    let op_ins = &ins.ins;
                    let con_index = Instructions::read_uint16(op_ins, ip + 1) as usize;
                    let con_obj = &self.constants[con_index].clone();
                    self.push(con_obj);

                    //println!("{con_index:?}");
                    //ip += 2;
                    self.adv_ip(2);
                }
                code::Opcode::Pop => {
                    self.last_popped = Rc::new(self.pop());
                }
                code::Opcode::Add
                | code::Opcode::Sub
                | code::Opcode::Mul
                | code::Opcode::Div
                | code::Opcode::Mod => self.exe_binary_op(op),

                code::Opcode::True => self.push(&TRUE),
                code::Opcode::False => self.push(&FALSE),
                code::Opcode::Equal | code::Opcode::NotEqual | code::Opcode::GT => {
                    self.exe_comparison(op)
                }
                code::Opcode::Bang => self.exe_bang_op(),
                code::Opcode::Minus => self.exe_pref_minux(),
                code::Opcode::Null => self.push(&NULL),
                code::Opcode::SetGlobal => {
                    let gi = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);
                    //self.globals[gi] = self.pop()
                    let pop_item = Rc::new(self.pop());
                    self.globals.push_value(gi, pop_item);
                }
                code::Opcode::GetGlobal => {
                    let gi = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    //self.push(&self.globals[gi].clone())
                    self.push(&self.globals.get_value(gi))
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
                    self.push(&arr);
                }

                code::Opcode::Hash => {
                    let num_of_elms = code::Instructions::read_uint16(&ins.ins, ip + 1) as usize;
                    //ip += 2;
                    self.adv_ip(2);

                    let hash = self.build_hash(self.sp - num_of_elms, self.sp);
                    self.sp -= num_of_elms;

                    self.push(&hash)
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
                    self.push(&rvalue)
                }
                code::Opcode::Return => {
                    let frm = self.pop_frame();
                    self.sp = frm.borrow().get_bp() as usize - 1;
                    //(*frm).borrow().get_bp() as usize - 1; //frm.as_ref().borrow().get_bp() as usize -1; //frm.bp as usize - 1;
                    //self.pop();
                    self.push(&NULL);
                }
                code::Opcode::SetLocal => {
                    let local_index = code::Instructions::read_u8(&ins.ins[ip + 1..]);
                    self.adv_ip(1);
                    let frm = self.current_frame().borrow().get_bp();
                    //self.current_frame().as_ref().borrow().get_bp(); //self.current_frame().bp;
                    self.stack[(frm as usize) + (local_index as usize)] = self.pop()
                }
                code::Opcode::GetLocal => {
                    let local_index = code::Instructions::read_u8(&ins.ins[ip + 1..]) as usize;
                    self.adv_ip(1);
                    let frm_bp = self.current_frame().borrow().get_bp() as usize;
                    //self.current_frame().as_ref().borrow().get_bp() as usize; //self.current_frame().bp as usize;
                    let stack_obj = self.stack[frm_bp + local_index].clone();
                    self.push(&stack_obj);
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

                    self.push(&ccl.frees[f_index as usize])
                }

                code::Opcode::CurrentClosure => {
                    //                    let ccl = self.current_frame().cl.clone();
                    let ccl = Rc::clone(&self.current_frame().borrow().cl);
                    //Rc::clone(&self.current_frame().cl);
                    //&self.current_frame().as_ref().borrow().cl.clone();

                    self.push(&Object::Closure(ccl.to_owned()));
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
            fr.push(Rc::new(
                self.stack.get(self.sp - num_free + i).unwrap().to_owned(),
            ));
            i += 1;
        }

        self.sp -= num_free;

        let cls = &Object::Closure(Rc::new(Closure {
            fun: cf.clone(),
            frees: fr,
        }));
        self.push(cls);
    }

    fn call_func(&mut self, num_args: usize) {
        let stack_object = &self.stack[self.sp - 1 - num_args].clone();
        //println!("STACK-OBJ{:?}" , stack_object);
        //let Object::Closure(cf) = stack_object else{
        //                panic!("stack object is not compiled function")
        //            } ;
        let Object::Closure(cf) = stack_object else{
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
        let mut hp: HashMap<Rc<HashKey>, Rc<HashPair>> = HashMap::new();

        let mut i = start;

        while i < end {
            let k = Rc::new(self.stack[i].clone());
            let v = Rc::new(self.stack[i + 1].clone());
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
            code::Opcode::Equal => self.push(&bool_native_to_obj(right == left)),
            code::Opcode::NotEqual => self.push(&bool_native_to_obj(left != right)),
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
            code::Opcode::Equal => self.push(&bool_native_to_obj(lval == rval)),
            code::Opcode::GT => self.push(&bool_native_to_obj(lval > rval)),
            code::Opcode::NotEqual => self.push(&bool_native_to_obj(lval != rval)),

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

            let Object::String { token : _, value : lval } = left else{
                panic!("left object is not string")
            };

            let Object::String { token : _, value : rval } = right else{
                panic!("left object is not string")
            };

            self.push(&Object::String {
                token: None,
                value: lval + &rval,
            });

            //            self.exe_binary_op_str(op, left, right)
        }
    }

    fn exe_binary_op_number(&mut self, op: code::Opcode, left: Object, right: Object) {
        let Object::Number { token : _, value } = left else {
            panic!("not a number")
        };
        let lval = value;

        let Object::Number { token : _, value } = right else {
            panic!("rval is not a number")
        };

        let rval = value;
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

        self.push(&Object::Number { token: None, value });
    }

    fn push(&mut self, obj: &Object) {
        if self.sp >= STACK_SIZE {
            panic!("stack overflow");
        }
        if self.sp >= self.stack.len() {
            self.stack.push(obj.to_owned())
        } else {
            self.stack[self.sp] = obj.to_owned()
        }
        //self.stack[self.sp] = obj.clone();
        //self.stack.push(obj.to_owned());
        self.sp += 1;
    }

    fn pop(&mut self) -> Object {
        let ip = self.sp - 1;
        //let obj = &self.stack[ip];
        //
        if self.sp == self.stack.len() {
            self.sp -= 1;
            return self.stack.pop().expect("stack is empty");
        }

        //let obj = self.stack.pop().unwrap();
        let obj = &self.stack[ip];
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
