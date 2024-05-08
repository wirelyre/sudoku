use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sudoku::prepare;

pub fn criterion_benchmark(c: &mut Criterion) {
    let puzzle = [
        [0, 0, 0, 0, 0, 0, 0, 1, 0],
        [0, 0, 0, 0, 0, 2, 0, 0, 3],
        [0, 0, 0, 4, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 5, 0, 0],
        [4, 0, 1, 6, 0, 0, 0, 0, 0],
        [0, 0, 7, 1, 0, 0, 0, 0, 0],
        [0, 5, 0, 0, 0, 0, 2, 0, 0],
        [0, 0, 0, 0, 8, 0, 0, 4, 0],
        [0, 3, 0, 9, 1, 0, 0, 0, 0],
    ];
    c.bench_function("solve 17", |b| {
        b.iter(|| prepare(black_box(&puzzle)).unwrap())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
