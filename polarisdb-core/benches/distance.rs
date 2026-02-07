//! Benchmarks for distance metric implementations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use polarisdb_core::distance::{cosine_distance, dot_product, euclidean_distance};
use rand::Rng;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

fn bench_euclidean(c: &mut Criterion) {
    let mut group = c.benchmark_group("euclidean_distance");

    for dim in [128, 384, 768, 1536].iter() {
        let a = generate_random_vector(*dim);
        let b = generate_random_vector(*dim);

        group.throughput(Throughput::Elements(*dim as u64));
        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |bench, _| {
            bench.iter(|| euclidean_distance(black_box(&a), black_box(&b)))
        });
    }

    group.finish();
}

fn bench_cosine(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine_distance");

    for dim in [128, 384, 768, 1536].iter() {
        let a = generate_random_vector(*dim);
        let b = generate_random_vector(*dim);

        group.throughput(Throughput::Elements(*dim as u64));
        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |bench, _| {
            bench.iter(|| cosine_distance(black_box(&a), black_box(&b)))
        });
    }

    group.finish();
}

fn bench_dot_product(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_product");

    for dim in [128, 384, 768, 1536].iter() {
        let a = generate_random_vector(*dim);
        let b = generate_random_vector(*dim);

        group.throughput(Throughput::Elements(*dim as u64));
        group.bench_with_input(BenchmarkId::from_parameter(dim), dim, |bench, _| {
            bench.iter(|| dot_product(black_box(&a), black_box(&b)))
        });
    }

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    use polarisdb_core::{BruteForceIndex, DistanceMetric, Payload};

    let mut group = c.benchmark_group("brute_force_search");

    for num_vectors in [1000, 10000, 50000].iter() {
        let dim = 384;
        let mut index = BruteForceIndex::new(DistanceMetric::Cosine, dim);

        // Insert vectors
        for i in 0..*num_vectors {
            let vector = generate_random_vector(dim);
            let payload = Payload::new().with_field("id", i as i64);
            index.insert(i as u64, vector, payload).unwrap();
        }

        let query = generate_random_vector(dim);

        group.throughput(Throughput::Elements(*num_vectors as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_vectors),
            num_vectors,
            |bench, _| bench.iter(|| index.search(black_box(&query), 10, None)),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_euclidean,
    bench_cosine,
    bench_dot_product,
    bench_search
);
criterion_main!(benches);
