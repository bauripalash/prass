use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pras::{lexer, parser};

fn parser_bench(s: &str) {
    let l = lexer::Lexer::new(s);
    let mut p = parser::Parser::new(l);
    let prog = p
        .parse_program()
        .expect("parser error on fibonacci benchmark");

    assert_eq!(prog.to_string() , "PROG[let<id<fib|false>:func(id<x|false>:blk<if(inf(ident(x)==(0)):blk<ret<(0)>;>:blk<if(inf(ident(x)==(1)):blk<ret<(1)>;>:blk<ret<inf(call(ident(fib):inf(ident(x)-(1)),)+call(ident(fib):inf(ident(x)-(2)),))>;>);>);>)>;call(ident(fib):(10),);]");

    //while !l.is_at_eof() {
    //    l.next_token();
    //}
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parser_if_else", |b| {
        b.iter(|| {
            parser_bench(black_box(
                "
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
    #dekhao(fib(22),1,2,3,4)",
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
