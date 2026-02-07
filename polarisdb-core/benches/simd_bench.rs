use criterion::{black_box, criterion_group, criterion_main, Criterion};
use polarisdb_core::distance::{dot_product, euclidean_distance_squared};
use rand::Rng;

fn bench_distance(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    
    // Benchmark with typical embedding sizes
    let dimensions = vec![128, 384, 768, 1536]; // e.g., BERT, OpenAI embeddings

    let mut group = c.benchmark_group("Distance Metrics");

    for dim in dimensions {
        let a: Vec<f32> = (0..dim).map(|_| rng.gen()).collect();
        let b: Vec<f32> = (0..dim).map(|_| rng.gen()).collect();

        group.bench_with_input(
            format!("Dot Product (dim={})", dim),
            &(a.clone(), b.clone()),
            |b, (v1, v2)| {
                b.iter(|| dot_product(black_box(v1), black_box(v2)))
            },
        );

        group.bench_with_input(
            format!("Euclidean Squared (dim={})", dim),
            &(a.clone(), b.clone()),
            |b, (v1, v2)| {
                b.iter(|| euclidean_distance_squared(black_box(v1), black_box(v2)))
            },
        );

        group.bench_with_input(
            format!("Cosine (dim={})", dim),
            &(a.clone(), b.clone()),
            |b, (v1, v2)| {
                use polarisdb_core::distance::cosine_distance;
                b.iter(|| cosine_distance(black_box(v1), black_box(v2)))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_distance);
criterion_main!(benches);
