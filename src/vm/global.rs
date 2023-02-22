use std::{cell::RefCell, rc::Rc};

use crate::obj::Object;

const GLOBALS_SIZE: usize = 1024; //Change

#[derive(Debug)]
pub struct GlobalStack {
    pub globals: Vec<Rc<Object>>,
    pub len: usize,
}

impl GlobalStack {
    pub fn new() -> Self {
        Self {
            globals: Vec::with_capacity(GLOBALS_SIZE),
            len: 0,
        }
    }
    pub fn push_value(&mut self, index: usize, obj: Rc<Object>) {
        if index >= self.len {
            self.globals.push(obj);
            self.len += 1;
        } else {
            //self.globals.borrow_mut()[index] = obj;
            //let ptr = self.globals.as_ptr();
            unsafe {
                //(*ptr)[index] = obj;

                _ = std::mem::replace(self.globals.get_unchecked_mut(index), obj);
            }
        }
    }

    pub fn get_value(&self, index: usize) -> Rc<Object> {
        if index >= self.len || index > GLOBALS_SIZE {
            Rc::new(Object::Null)
        } else {
            unsafe { Rc::clone(&self.globals.get_unchecked(index)) }
        }
    }
}
