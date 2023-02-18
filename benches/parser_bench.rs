use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pras::{
    lexer,
    parser::{self, Parser},
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

fn parser_bench(p: &mut Parser) {
    p.parse_program().expect("parser_error"); //while !l.is_at_eof() {
                                              //    l.next_token();
                                              //}
}

fn criterion_benchmark(c: &mut Criterion) {
    let lx = lexer::Lexer::new(INPUT);
    let mut p = parser::Parser::new(lx);
    c.bench_function("parse_fib_10", |b| {
        b.iter(|| parser_bench(black_box(&mut p)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
