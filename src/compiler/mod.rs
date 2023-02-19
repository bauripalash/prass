use crate::{
    ast,
    obj::{CompFunc, Object},
    token::{Token, TokenType},
};
use std::{cell::RefCell, rc::Rc};

use self::{
    code::{get_def, make_ins, u8_to_op, Bytecode, Instructions, Opcode},
    symtab::{Symbol, Table},
};

pub mod code;
pub mod symtab;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedIns {
    pub opcode: code::Opcode,
    pub pos: usize,
}

impl EmittedIns {
    pub const fn new() -> Self {
        Self {
            opcode: code::Opcode::Dummy,
            pos: 0,
        }
    }
}

impl Default for EmittedIns {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompScope {
    pub ins: code::Instructions,
    last_ins: EmittedIns,
    prev_ins: EmittedIns,
}

#[derive(Debug, Clone)]
pub struct Compiler {
    pub symtab: Rc<RefCell<symtab::Table>>,
    constants: Vec<Rc<Object>>,
    scopes: Vec<CompScope>,
    scope_index: usize,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mainscope = CompScope {
            ins: code::Instructions::new(),
            last_ins: EmittedIns::new(),
            prev_ins: EmittedIns::new(),
        };

        Self {
            symtab: Rc::new(RefCell::new(symtab::Table::new())),
            constants: Vec::new(),
            scopes: vec![mainscope],
            scope_index: 0,
        }
    }

    pub fn current_ins(&self) -> &code::Instructions {
        &self.scopes[self.scope_index].ins
    }

    pub fn current_ins_mut(&mut self) -> &mut code::Instructions {
        &mut self.scopes[self.scope_index].ins
    }

    pub fn compile(&mut self, node: ast::Program) -> Bytecode {
        for s in node.stmts {
            self.compile_stmt(&s)
        }

        self.bytecode()
    }

    fn sym_define(&mut self, name: &str) -> Rc<Symbol> {
        self.symtab.borrow_mut().define(name)
    }

    fn sym_resolve(&mut self, name: &str) -> Result<Rc<Symbol>, bool> {
        self.symtab.borrow_mut().resolve(name.to_string())
    }

    fn sym_define_fun(&mut self, name: &str) -> Rc<Symbol> {
        self.symtab.borrow_mut().define_func(name.to_string())
    }

    fn sym_free_syms(&self) -> Vec<Rc<Symbol>> {
        self.symtab.borrow().free_syms.clone()
    }

    pub fn compile_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::LetStmt {
                token: _,
                name,
                value,
            } => {
                let sm = self.sym_define(&name.name);
                self.compiler_expr(value);

                match sm.scope {
                    symtab::Scope::Global => {
                        self.emit(Opcode::SetGlobal, Some(&vec![sm.index]));
                    }
                    symtab::Scope::Local => {
                        self.emit(Opcode::SetLocal, Some(&vec![sm.index]));
                    }
                    _ => {}
                };
            }
            ast::Stmt::ExprStmt { token: _, expr } => {
                self.compiler_expr(expr);
                self.emit(Opcode::Pop, None);
            }
            ast::Stmt::BlockStmt { token: _, stmts } => {
                for s in stmts {
                    self.compile_stmt(s)
                }
            }
            ast::Stmt::ReturnStmt { token: _, rval } => {
                self.compiler_expr(rval);
                self.emit(Opcode::ReturnValue, None);
            }
            ast::Stmt::ShowStmt { token: _, value } => {
                //println!("{:?}" , value);
                for v in value.iter() {
                    self.compiler_expr(v)
                }
                self.emit(Opcode::Show, Some(&vec![value.len()]));
            } // _ => {}
        }
    }

    pub fn compiler_expr(&mut self, expr: &ast::Expr) {
        match expr {
            ast::Expr::IdentExpr { token: _, value } => {
                let sm = self.sym_resolve(value);

                if let Ok(s) = sm {
                    self.load_symbol(&s);
                } else {
                    panic!("undefined variable {value}");
                }
            }
            ast::Expr::StringExpr { token, value } => {
                let sl = Rc::new(Object::String {
                    token: Some(token.to_owned()),
                    value: value.to_string(),
                });
                let con = self.add_const(sl);
                //println!("{con}");
                self.emit(Opcode::Const, Some(&vec![con]));
            }
            ast::Expr::NumExpr {
                token,
                value,
                is_int: _,
            } => {
                let num = Rc::new(Object::Number {
                    token: Some(token.to_owned()),
                    value: value.clone(),
                });
                let con = self.add_const(num);
                self.emit(Opcode::Const, Some(&vec![con]));
            }
            ast::Expr::ArrayExpr { token: _, elems } => {
                for el in elems {
                    self.compiler_expr(el)
                }
                self.emit(Opcode::Array, Some(&vec![elems.len()]));
            }
            ast::Expr::BoolExpr { token: _, value } => {
                if *value {
                    self.emit(Opcode::True, None);
                } else {
                    self.emit(Opcode::False, None);
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

                let jntpos = self.emit(Opcode::JumpNotTruthy, Some(&vec![9999]));
                self.compile_stmt(trueblock);

                if self.is_last_ins(&Opcode::Pop) {
                    self.remove_last_pop();
                }

                let jmppos = self.emit(Opcode::Jump, Some(&vec![9999]));
                let after_tb_pos = self.current_ins().ins.len();
                self.change_operand(jntpos, after_tb_pos);

                if let Some(eb) = elseblock {
                    self.compile_stmt(eb);
                    if self.is_last_ins(&Opcode::Pop) {
                        self.remove_last_pop();
                    }
                } else {
                    self.emit(Opcode::Null, None);
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
                self.emit(Opcode::Hash, Some(&vec![p.len() * 2]));
            }
            ast::Expr::IndexExpr {
                token: _,
                left,
                index,
            } => {
                self.compiler_expr(left);
                self.compiler_expr(index);
                self.emit(Opcode::Index, None);
            }

            ast::Expr::FuncExpr(f) => {
                self.enter_scope();
                if !f.name.is_empty() {
                    self.sym_define_fun(&f.name);
                }
                let fun_params = f.params.to_vec();

                for p in &fun_params {
                    self.sym_define(&p.name);
                }
                self.compile_stmt(&f.body);
                if self.is_last_ins(&Opcode::Pop) {
                    self.replace_last_pop_with_return()
                }

                if !self.is_last_ins(&Opcode::ReturnValue) {
                    self.emit(Opcode::Return, None);
                }
                let free_syms = self.sym_free_syms();
                let num_locals = self.symtab.borrow().numdef;
                let ins = self.leave_scope();

                for s in &free_syms {
                    self.load_symbol(s);
                }

                let cmp_fn = Rc::new(Object::Compfunc(Rc::new(CompFunc {
                    fnin: Rc::new(ins),
                    num_locals,
                    num_params: fun_params.len(),
                })));
                let con = self.add_const(cmp_fn);
                self.emit(Opcode::Closure, Some(&vec![con, free_syms.len()]));
            }

            ast::Expr::CallExpr {
                token: _,
                func,
                args,
            } => {
                self.compiler_expr(func);
                for arg in args {
                    self.compiler_expr(arg)
                }
                self.emit(Opcode::Call, Some(&vec![args.len()]));
            }

            _ => {}
        }
    }

    fn load_symbol(&mut self, sym: &Symbol) {
        match sym.scope {
            symtab::Scope::Global => self.emit(Opcode::GetGlobal, Some(&vec![sym.index])),
            symtab::Scope::Local => self.emit(Opcode::GetLocal, Some(&vec![sym.index])),
            symtab::Scope::Free => self.emit(Opcode::GetFree, Some(&vec![sym.index])),
            symtab::Scope::Func => self.emit(Opcode::CurrentClosure, None),
        };
    }

    fn replace_last_pop_with_return(&mut self) {
        let lastpos = self.scopes[self.scope_index].last_ins.pos;

        self.replace_ins(lastpos, code::make_ins(Opcode::ReturnValue, &[]));
        self.scopes[self.scope_index].last_ins.opcode = Opcode::ReturnValue;
    }

    pub fn compiler_prefix_expr(&mut self, right: &ast::Expr, op: &Token) {
        self.compiler_expr(right);

        match op.ttype {
            TokenType::BANG => self.emit(Opcode::Bang, None),
            TokenType::Minus => self.emit(Opcode::Minus, None),
            _ => panic!("prefix unknonw operator -> {} ", op.literal),
        };
    }

    pub fn replace_ins(&mut self, pos: usize, new_ins: Vec<u8>) {
        let mut i = 0;
        while i < new_ins.len() {
            self.current_ins_mut().ins[pos + i] = new_ins[i];
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
            TokenType::Plus => self.emit(Opcode::Add, None),
            TokenType::Minus => self.emit(Opcode::Sub, None),
            TokenType::Mul => self.emit(Opcode::Mul, None),
            TokenType::Div => self.emit(Opcode::Div, None),
            TokenType::MOD => self.emit(Opcode::Mod, None),
            TokenType::GT => self.emit(Opcode::GT, None),
            TokenType::EqEq => self.emit(Opcode::Equal, None),
            TokenType::NotEq => self.emit(Opcode::NotEqual, None),
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
        let prev = &self.scopes[self.scope_index].last_ins; //self.last_ins.clone();
        let last = EmittedIns { opcode: op, pos };
        self.scopes[self.scope_index].prev_ins = prev.clone();
        self.scopes[self.scope_index].last_ins = last;
    }

    fn is_last_ins(&self, op: &Opcode) -> bool {
        if self.current_ins().ins.is_empty() {
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

    fn add_const(&mut self, obj: Rc<Object>) -> usize {
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

    pub fn enter_scope(&mut self) {
        let scope = CompScope {
            ins: code::Instructions::new(),
            last_ins: EmittedIns::new(),
            prev_ins: EmittedIns::new(),
        };

        //        self.symtab = Rc::new(
        //                RefCell::new(Table::new_enclosed(Rc::new(self.symtab.borrow())))
        //            );
        //
        self.symtab = Rc::new(RefCell::new(Table::new_enclosed(self.symtab.borrow())));
        self.scopes.push(scope);

        self.scope_index += 1;
    }

    pub fn leave_scope(&mut self) -> code::Instructions {
        let ins = self.current_ins().clone();
        self.scopes = self.scopes[..self.scopes.len() - 1].to_vec();
        self.scope_index -= 1;

        let x = self.symtab.borrow().get_outer_no_check();
        self.symtab = Rc::new(RefCell::new(x));

        ins
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: Rc::new(self.current_ins().clone()),
            constants: self.constants.clone(),
        }
    }
}
