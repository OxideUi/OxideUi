// Benchmarks for layout engine

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Replace this with real layout engine benchmarks once available.
fn bench_layout_basic(c: &mut Criterion) {
    c.bench_function("layout_basic_compute", |b| {
        b.iter(|| {
            // Simulate a small layout computation workload
            let mut acc = 0.0_f32;
            for i in 0..1024 {
                // black_box to avoid compiler optimizing the loop away
                acc += (black_box(i as f32).sin() * 0.5 + 0.5) * 3.14159;
            }
            black_box(acc);
        })
    });
}

criterion_group!(benches, bench_layout_basic);
criterion_main!(benches);
