use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pras::{
    compiler::{code::Bytecode, Compiler},
    lexer, parser,
    vm::Vm,
};

pub static INPUT: &str = "
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
    fib(10)
    #dekhao(fib(22),1,2,3,4)";

fn vm_bench(bc: &Bytecode) {
    let mut vm = Vm::new(bc.to_owned());
    vm.run();
    _ = vm.last_pop()

    //while !l.is_at_eof() {
    //    l.next_token();
    //}
}

fn criterion_benchmark(c: &mut Criterion) {
    let l = lexer::Lexer::new(INPUT);
    let mut p = parser::Parser::new(l);
    let prog = p
        .parse_program()
        .expect("parser error on fibonacci benchmark");
    let mut com = Compiler::new();
    let bc = com.compile(prog);
    c.bench_function("vm_fib_10", |b| b.iter(|| vm_bench(black_box(&bc))));
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
