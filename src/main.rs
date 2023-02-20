use std::process::exit;

use pras::compiler::Compiler;
//use pras::{compiler::Compiler, lexer::Lexer, parser::Parser, vm::Vm};
use pras::lexer::Lexer;
use pras::parser::Parser;
use pras::vm::Vm;

fn main() {
    let fib = "
    dhori fib = ekti kaj(x)
        jodi (x == 0) tahole
            ferao(0)
        nahole 
            jodi (x==1) tahole
                ferao(1)
            nahole 
                ferao(fib(x-1) + fib(x-2))
            sesh 
        sesh 
    sesh
    dekhao(fib(25))
        ";

    //let a = Lexer::new("ekti kaj(x) x+1 sesh(2)");
    //let a = Lexer::new("ekti kaj(x) 100 + x * 9 sesh(9) ");
    let a = Lexer::new(fib);
    let mut parser = Parser::new(a);
    let parsed_program = parser.parse_program();
    //println!("{}", parsed_program.expect());
    if let Ok(ast) = parsed_program {
        println!("parse done!");
        let mut comp = Compiler::new();
        comp.compile(ast);
        //println!("{}", comp.bytecode());

        let mut v = Vm::new(comp.bytecode());
        v.run();
        //println!("{:?} -> {:?}" , v.constants , v.stack);
        println!("{:?}", v.last_pop());
    } else {
        //for err in &parser.errors {
        //    println!("Err => {}", err.msg);
        //}
        parser.print_errorrs();
        //panic!("please fix parser errors");
        exit(1);
    }

    //  for item in &c.constants{
    //     println!("con->{item}");
    //}
    //let mut v = Vm::new(c);
    //v.run();
    //println!("result->{}", v.last_pop());
}

/*
use std::{rc::Rc, cell::RefCell};
const SS : usize = 10;
fn main() {
    let stack : Rc<RefCell<Vec<Rc<RefCell<String>>>>> = Rc::new(RefCell::new(Vec::with_capacity(SS)));
    let x = ["a" , "b" , "c"];

    for item in x{
        (stack.borrow_mut()).push(Rc::new(RefCell::new(item.to_string())))
    }


    unsafe{

    let y = &stack.borrow_mut()[1];
        let z = y.as_ptr();
        *z= "l".to_string();
            //Rc::new(RefCell::new("l".to_string()));

    }


    println!("{:?}" , stack);
}
*/
