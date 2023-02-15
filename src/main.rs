use pras::{compiler::Compiler, lexer::Lexer, parser::Parser, vm::Vm};

fn main() {
   /* let src_x = "
 dhori global = 55

 ekti kaj()
    dhori a = 66

    ekti kaj()
        dhori b = 77
        
        ekti kaj()
            dhori c = 88 
            global + a + b + c 
        sesh
    sesh
sesh

";*/

    let src = "
    dhori fib = ekti kaj(x)
        jodi (x == 0) tahole
            ferao(0)
        nahole 
            jodi (x==1) tahole 
                ferao(1)
            nahole 
                fib(x-1) + fib(x-2)
            sesh 
        sesh 
    sesh

    fib(22)
        ";

  /*  let src_z = "dhori newadder = ekti kaj(a,b)
            ekti kaj(c)
                a+b+c 
            sesh
            sesh
            
            dhori adder = newadder(1,2)
            adder(8)
            ";*/
    //    let src = "dhori a = ekti kaj() 1 sesh
    //        a()";
    //   let src = "ekti kaj() 1; 2 sesh";
    //let src = "jodi (true) tahole 1 nahole sesh";

    let a = Lexer::new(src);
    let mut parser = Parser::new(a);
    let parsed_program = parser.parse_program();

    if !parser.errors.is_empty() {
        for err in &parser.errors {
            println!("Err => {}", err.msg);
        }
        panic!("please fix parser errors");
    }

    let mut cm = Compiler::new();
    let c = cm.compile(parsed_program);
        println!("{}", c.instructions);
    //  for item in &c.constants{
    //     println!("con->{item}");
    //}
    let mut v = Vm::new(c);
    v.run();
    println!("result->{}", v.last_pop());
}
