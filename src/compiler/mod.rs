use std::rc::Rc;

use crate::{
    ast,
    obj::Object,
    token::{Token, TokenType},
};

use self::code::{get_def, make_ins, u8_to_op, Bytecode, Instructions, Opcode};

pub mod code;
pub mod symtab;

#[derive(Debug, Clone,PartialEq, Eq)]
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

#[derive(Debug , Clone , PartialEq , Eq)]
pub struct CompScope{
    pub ins : code::Instructions,
    last_ins: EmittedIns,
    prev_ins: EmittedIns,
}

#[derive(Debug, Clone)]
pub struct Compiler {
    symtab: symtab::Table,
    //instructions: code::Instructions,
    constants: Vec<Object>,
    scopes : Vec<CompScope>,
    scope_index : usize,
    //last_ins: EmittedIns,
    //prev_ins: EmittedIns,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mainscope = CompScope{
            ins : code::Instructions::new(),
            last_ins : EmittedIns::new(),
            prev_ins : EmittedIns::new(),
        };
        Self {
            symtab: symtab::Table::new(),
            //instructions: code::Instructions::new(),
            constants: Vec::new(),
            
            scopes : vec![mainscope],
            scope_index : 0,
//            last_ins: EmittedIns::new(),
  //          prev_ins: EmittedIns::new(),
        }
    }

    pub fn current_ins(&self) -> code::Instructions {
        self.scopes[self.scope_index].ins.clone()
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
            },
            ast::Stmt::ReturnStmt { token : _, rval } => {
                self.compiler_expr(rval);
                self.emit(Opcode::OpReturnValue, None);
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
                let after_tb_pos = self.current_ins().ins.len();
                self.change_operand(jntpos, after_tb_pos);

                if let Some(eb) = elseblock {
                    self.compile_stmt(eb);
                    if self.is_last_ins(&Opcode::OpPop) {
                        self.remove_last_pop();
                    }
                } else {
                    self.emit(Opcode::OpNull, None);
                }

                let after_eb_pos = self.current_ins().ins.len();
                self.change_operand(jmppos, after_eb_pos);
            }

            ast::Expr::HashExpr { token: _, pairs } => {
                let mut p = pairs.clone();
                p.sort_by_key(|(k, _)| k.to_string());
                for (k, v) in &p {
                    self.compiler_expr(k);
                    self.compiler_expr(v)
                }
                self.emit(Opcode::OpHash, Some(&vec![p.len() * 2]));
            }
            ast::Expr::IndexExpr {
                token: _,
                left,
                index,
            } => {
                self.compiler_expr(left);
                self.compiler_expr(index);
                self.emit(Opcode::OpIndex, None);
            },

            ast::Expr::FuncExpr { token : _, params, body } => {
                self.enter_scope();
                self.compile_stmt(body);
                if self.is_last_ins(&Opcode::OpPop){
                    self.replace_last_pop_with_return()

                }
                let ins = self.leave_scope();

                let cmp_fn = Object::Compfunc { ins: Rc::new(ins) };
                let con = self.add_const(cmp_fn); 
                self.emit(Opcode::OpConst, Some(&vec![con]));
            }

            _ => {}
        }
    }

    fn replace_last_pop_with_return(&mut self) {
        let lastpos = self.scopes[self.scope_index].last_ins.pos;
    
        self.replace_ins(lastpos, code::make_ins(Opcode::OpReturnValue, &[]));
        self.scopes[self.scope_index].last_ins.opcode = Opcode::OpReturnValue;
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
            self.current_ins().ins[pos+i] = new_ins[i];
//            self.instructions.ins[pos + i] = new_ins[i];
            i += 1;
        }
    }

    pub fn change_operand(&mut self, pos: usize, operand: usize) {
        let op = u8_to_op(self.current_ins().ins[pos]);
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
        let prev =  &self.scopes[self.scope_index].last_ins; //self.last_ins.clone();
        let last = EmittedIns { opcode: op, pos };
        self.scopes[self.scope_index].prev_ins = prev.clone();
        self.scopes[self.scope_index].last_ins = last;
    }

    fn is_last_ins(&self, op: &Opcode) -> bool {
        if self.current_ins().ins.len() == 0{
            return false;
        }
        self.scopes[self.scope_index].last_ins.opcode == *op
    }

    fn remove_last_pop(&mut self) {
        //self.instructions.ins = self.instructions.ins[..self.last_ins.pos].to_vec();
        //self.last_ins = self.prev_ins.clone();
        let last = self.scopes[self.scope_index].last_ins.clone();
        let prev = self.scopes[self.scope_index].prev_ins.clone();

        let old = self.current_ins().clone();
        let new = &old.ins[..last.pos];

        self.scopes[self.scope_index].ins.ins = new.to_vec();
        self.scopes[self.scope_index].last_ins = prev;
    }

    fn add_const(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    pub fn add_inst(&mut self, ins: Instructions) -> usize {
        let pos_of_new_ins = self.current_ins().ins.len();
        let mut cloned_ins = self.current_ins().clone();
        cloned_ins.add_ins(ins.ins);
        self.scopes[self.scope_index].ins = cloned_ins;
        

        
        
        //self.instructions.add_ins(ins.ins);
        pos_of_new_ins
    }

    pub fn enter_scope(&mut self){
        let scope = CompScope {
            ins : code::Instructions::new(),
            last_ins : EmittedIns::new(),
            prev_ins : EmittedIns::new(),
        };

        self.scopes.push(scope);
        self.scope_index += 1;
    }

    pub fn leave_scope(&mut self) -> code::Instructions {
        let ins = self.current_ins();
        self.scopes = self.scopes[..self.scopes.len()-1].to_vec();
        self.scope_index -= 1;

        ins
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.current_ins().clone(),
            constants: self.constants.clone(),
        }
    }
}
