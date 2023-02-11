use std::rc::Rc;

use crate::{
    ast::{self, Stmt},
    obj::Object,
    token::TokenType,
};

use self::code::{make_ins, Bytecode, Instructions, Opcode};

pub mod code;

#[derive(Debug, Clone)]
pub struct Compiler {
    instructions: code::Instructions,
    constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: code::Instructions::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile(&mut self, node: ast::Program) -> Bytecode {
        for s in node.stmts {
            self.compile_stmt(&s)
        }

        self.bytecode()
    }

    pub fn compile_stmt(&mut self, stmt: &ast::Stmt) {
        //println!("{stmt}");

        match stmt {
            ast::Stmt::ExprStmt { token: _, expr } => self.compiler_expr(expr),
            _ => {}
        }
    }

    pub fn compiler_expr(&mut self, expr: &ast::Expr) {
        match expr {
            ast::Expr::NumExpr {
                token,
                value,
                is_int: _,
            } => {
                let num = Object::Number {
                    token: Rc::new(token.clone()),
                    value: value.clone(),
                };
                let con = self.add_const(num.clone());
                //println!("{con}");
                self.emit(Opcode::OpConst, Some(&vec![con]));
            }
            ast::Expr::InfixExpr {
                token: _,
                left,
                op,
                right,
            } => {
                //println!("{expr:?} -> ");
                self.compiler_expr(&left);
                self.compiler_expr(&right);

                match op.ttype {
                    TokenType::Plus => {
                        //println!("op +");
                        self.emit(Opcode::OpAdd, None);
                    }

                    _ => {
                        panic!("unknown operator {}", op.literal);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn emit(&mut self, op: Opcode, operands: Option<&Vec<usize>>) -> usize {
        let ins: Vec<u8>;
        if let Some(o) = operands {
            ins = make_ins(op, o);
        } else {
            ins = make_ins(op, &vec![]);
        }

        return self.add_inst(Instructions { ins });
    }

    fn add_const(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    pub fn add_inst(&mut self, ins: Instructions) -> usize {
        let pos_of_new_ins = self.instructions.ins.len();
        self.instructions.add_ins(ins.ins);
        pos_of_new_ins
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }
}
