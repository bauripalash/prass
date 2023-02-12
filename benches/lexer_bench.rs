use criterion::{criterion_group, criterion_main, Criterion, black_box};
use pras::lexer;


fn lexer_bench(s : &str){
    let mut l = lexer::Lexer::new(s);
    while !l.is_at_eof() {
        l.next_token();
    }
}

fn criterion_benchmark(c : &mut Criterion){
   c.bench_function("lexer_let_1p2", |b| b.iter(|| lexer_bench(black_box("let a = 1+2"))));
}

criterion_group!(benches , criterion_benchmark);
criterion_main!(benches);
