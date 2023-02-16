use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pras::{lexer, parser};

fn parser_bench(s: &str) {
    let l = lexer::Lexer::new(s);
    let mut p = parser::Parser::new(l);
    p.parse_program();

    //while !l.is_at_eof() {
    //    l.next_token();
    //}
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parser_if_else", |b| {
        b.iter(|| parser_bench(black_box("jodi (true) tahole 1 nahole 2 sesh")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
