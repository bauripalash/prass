use std::rc::Rc;

use crate::{
    ast,
    obj::Object,
    token::{Token, TokenType},
};

use self::code::{get_def, make_ins, u8_to_op, Bytecode, Instructions, Opcode};

pub mod code;
pub mod symtab;

#[derive(Debug, Clone)]
pub struct EmittedIns {
    pub opcode: code::Opcode,
    pub pos: usize,
}

impl EmittedIns {
    pub const fn new() -> Self {
        Self {
            opcode: code::Opcode::OpDummy,
            pos: 0,
        }
    }
}

impl Default for EmittedIns {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Compiler {
    symtab: symtab::Table,
    instructions: code::Instructions,
    constants: Vec<Object>,
    last_ins: EmittedIns,
    prev_ins: EmittedIns,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            symtab: symtab::Table::new(),
            instructions: code::Instructions::new(),
            constants: Vec::new(),
            last_ins: EmittedIns::new(),
            prev_ins: EmittedIns::new(),
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
            ast::Stmt::LetStmt {
                token: _,
                name,
                value,
            } => {
                self.compiler_expr(value);

                let sm = self.symtab.define(name.name.clone());
                self.emit(Opcode::OpSetGlobal, Some(&vec![sm.index]));

                //self.emit(Opcode::OpPop, None);
            }
            ast::Stmt::ExprStmt { token: _, expr } => {
                self.compiler_expr(expr);
                self.emit(Opcode::OpPop, None);
            }
            ast::Stmt::BlockStmt { token: _, stmts } => {
                for s in stmts {
                    self.compile_stmt(s)
                }
            }

            _ => {}
        }
    }

    pub fn compiler_expr(&mut self, expr: &ast::Expr) {
        match expr {
            ast::Expr::IdentExpr { token: _, value } => {
                let sm = self.symtab.resolve(value.clone());
                if let Ok(s) = sm {
                    self.emit(Opcode::OpGetGlobal, Some(&vec![s.index]));
                } else {
                    panic!("undefined variable {value}");
                }
            }
            ast::Expr::StringExpr { token, value } => {
                let sl = Object::String {
                    token: Some(Rc::new(token.clone())),
                    value: value.to_string(),
                };
                let con = self.add_const(sl);
                //println!("{con}");
                self.emit(Opcode::OpConst, Some(&vec![con]));
            }
            ast::Expr::NumExpr {
                token,
                value,
                is_int: _,
            } => {
                let num = Object::Number {
                    token: Some(Rc::new(token.clone())),
                    value: value.clone(),
                };
                let con = self.add_const(num);
                self.emit(Opcode::OpConst, Some(&vec![con]));
            }
            ast::Expr::ArrayExpr { token: _, elems } => {
                for el in elems {
                    self.compiler_expr(el)
                }
                self.emit(Opcode::OpArray, Some(&vec![elems.len()]));
            }
            ast::Expr::BoolExpr { token: _, value } => {
                if *value {
                    self.emit(Opcode::OpTrue, None);
                } else {
                    self.emit(Opcode::OpFalse, None);
                }
            }
            ast::Expr::InfixExpr {
                token: _,
                left,
                op,
                right,
            } => self.compile_infix_expr(left, right, op),
            ast::Expr::PrefixExpr {
                token: _,
                op,
                right,
            } => self.compiler_prefix_expr(right, op),
            ast::Expr::IfExpr {
                token: _,
                cond,
                trueblock,
                elseblock,
            } => {
                self.compiler_expr(cond);

                let jntpos = self.emit(Opcode::OpJumpNotTruthy, Some(&vec![9999]));
                self.compile_stmt(trueblock);

                if self.is_last_ins(&Opcode::OpPop) {
                    self.remove_last_pop();
                }

                let jmppos = self.emit(Opcode::OpJump, Some(&vec![9999]));
                let after_tb_pos = self.instructions.ins.len();
                self.change_operand(jntpos, after_tb_pos);

                if let Some(eb) = elseblock {
                    self.compile_stmt(eb);
                    if self.is_last_ins(&Opcode::OpPop) {
                        self.remove_last_pop();
                    }
                } else {
                    self.emit(Opcode::OpNull, None);
                }

                let after_eb_pos = self.instructions.ins.len();
                self.change_operand(jmppos, after_eb_pos);
            }

            _ => {}
        }
    }

    pub fn compiler_prefix_expr(&mut self, right: &ast::Expr, op: &Token) {
        self.compiler_expr(right);

        match op.ttype {
            TokenType::BANG => self.emit(Opcode::OpBang, None),
            TokenType::Minus => self.emit(Opcode::OpMinus, None),
            _ => panic!("prefix unknonw operator -> {} ", op.literal),
        };
    }

    pub fn replace_ins(&mut self, pos: usize, new_ins: Vec<u8>) {
        let mut i = 0;
        while i < new_ins.len() {
            self.instructions.ins[pos + i] = new_ins[i];
            i += 1;
        }
    }

    pub fn change_operand(&mut self, pos: usize, operand: usize) {
        let op = u8_to_op(self.instructions.ins[pos]);
        let ins = make_ins(op, &[operand]);
        self.replace_ins(pos, ins);
    }

    pub fn compile_infix_expr(&mut self, left: &ast::Expr, right: &ast::Expr, op: &Token) {
        self.compiler_expr(left);
        self.compiler_expr(right);
        match op.ttype {
            TokenType::Plus => self.emit(Opcode::OpAdd, None),
            TokenType::Minus => self.emit(Opcode::OpSub, None),
            TokenType::Mul => self.emit(Opcode::OpMul, None),
            TokenType::Div => self.emit(Opcode::OpDiv, None),
            TokenType::MOD => self.emit(Opcode::OpMod, None),
            TokenType::GT => self.emit(Opcode::OpGT, None),
            TokenType::EqEq => self.emit(Opcode::OpEqual, None),
            TokenType::NotEq => self.emit(Opcode::OpNotEqual, None),
            _ => panic!("unknown operator -> {}", op.literal),
        };
    }

    pub fn emit(&mut self, op: Opcode, operands: Option<&Vec<usize>>) -> usize {
        let d = get_def(&op).op_width.len();
        let ins: Vec<u8>;

        if let Some(o) = operands {
            if d != o.len() {
                panic!(
                    "OpCode {op:?} operand does not match. W=>{d} G=>{}",
                    o.len()
                )
            }
            ins = make_ins(op, o);
        } else {
            if d > 0 {
                panic!("OpCode {op:?} operand does not match. W=>{d} G=>0")
            }

            ins = make_ins(op, &[]);
        }

        let pos = self.add_inst(Instructions { ins });

        self.set_last_ins(op, pos);

        pos
    }

    fn set_last_ins(&mut self, op: Opcode, pos: usize) {
        let prev = self.last_ins.clone();
        let last = EmittedIns { opcode: op, pos };
        self.prev_ins = prev;
        self.last_ins = last;
    }

    fn is_last_ins(&self, op: &Opcode) -> bool {
        self.last_ins.opcode == *op
    }

    fn remove_last_pop(&mut self) {
        self.instructions.ins = self.instructions.ins[..self.last_ins.pos].to_vec();
        self.last_ins = self.prev_ins.clone();
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
