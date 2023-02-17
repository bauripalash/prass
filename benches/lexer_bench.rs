use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pras::lexer;

pub static INPUT : &str = "
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

fn lexer_bench(s: &str) {
    let mut l = lexer::Lexer::new(s);
    while !l.is_at_eof() {
        _ = l.next_token();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lexer_let_1p2", |b| {
        b.iter(|| {
            lexer_bench(black_box(
              INPUT 
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
