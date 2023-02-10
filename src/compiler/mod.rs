use crate::{obj::Object, ast::{self, Stmt}};

use self::code::{Opcode, Instructions, Bytecode};

pub mod code;


#[derive(Debug , Clone)]
pub struct Compiler {
    instructions : code::Instructions ,
    constants : Vec<Object>,
}



impl Compiler {
    pub fn new() -> Self {
        Self { 
            instructions: code::Instructions::new(), 
            constants: Vec::new()
        }
    }

    pub fn compile(&mut self , node : ast::Program) -> Bytecode{
        self.bytecode()
    }

    pub fn compile_stmt(&mut self , stmt : &ast::Stmt){
        match stmt {
            ast::Stmt::ReturnStmt { token : _, rval : _ } => {
                
            }
            _=>{}
        }
    }

    pub fn emit(&mut self , op : Opcode , operands : &Vec<usize>) -> usize{
        let ins = code::make_ins(op, operands);
        0
    }

    pub fn add_inst(&mut self , ins : Instructions) {
        
    }

    pub fn  bytecode(&self) -> Bytecode{
        Bytecode { instructions: self.instructions.clone(), constants: self.constants.clone() }
    }
}
