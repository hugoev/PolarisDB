//! HNSW demo - comparing brute-force vs HNSW search performance.
//!
//! This example demonstrates how HNSW provides fast approximate
//! nearest neighbor search compared to brute-force exact search.

use polarisdb::prelude::*;
use std::time::Instant;

fn main() {
    println!("🚀 HNSW Performance Demo\n");

    // Parameters
    let num_vectors = 10_000;
    let dimension = 128;
    let k = 10;

    println!(
        "📊 Setup: {} vectors, {} dimensions",
        num_vectors, dimension
    );
    println!();

    // Generate random vectors
    println!("⏳ Generating {} random vectors...", num_vectors);
    let vectors: Vec<Vec<f32>> = (0..num_vectors)
        .map(|i| {
            (0..dimension)
                .map(|j| ((i * dimension + j) as f32 * 0.1).sin())
                .collect()
        })
        .collect();

    // Build brute-force index
    println!("🔨 Building brute-force index...");
    let start = Instant::now();
    let mut bf_index = BruteForceIndex::new(DistanceMetric::Euclidean, dimension);
    for (i, v) in vectors.iter().enumerate() {
        bf_index
            .insert(i as u64, v.clone(), Payload::new())
            .unwrap();
    }
    let bf_build_time = start.elapsed();
    println!("   ✅ Built in {:?}", bf_build_time);

    // Build HNSW index
    println!("🔨 Building HNSW index...");
    let config = HnswConfig {
        m: 16,
        m_max0: 32,
        ef_construction: 200,
        ef_search: 100,
    };
    let start = Instant::now();
    let mut hnsw_index = HnswIndex::new(DistanceMetric::Euclidean, dimension, config);
    for (i, v) in vectors.iter().enumerate() {
        hnsw_index
            .insert(i as u64, v.clone(), Payload::new())
            .unwrap();
    }
    let hnsw_build_time = start.elapsed();
    println!("   ✅ Built in {:?}", hnsw_build_time);
    println!();

    // Query
    let query: Vec<f32> = (0..dimension).map(|j| (j as f32 * 0.15).cos()).collect();

    // Brute-force search
    println!("🔍 Brute-force search (exact k={})...", k);
    let start = Instant::now();
    let bf_results = bf_index.search(&query, k, None);
    let bf_search_time = start.elapsed();
    println!("   ⏱️  Time: {:?}", bf_search_time);

    // HNSW search
    println!("🔍 HNSW search (approximate k={})...", k);
    let start = Instant::now();
    let hnsw_results = hnsw_index.search(&query, k, Some(200), None);
    let hnsw_search_time = start.elapsed();
    println!("   ⏱️  Time: {:?}", hnsw_search_time);
    println!();

    // Calculate recall
    let bf_ids: std::collections::HashSet<_> = bf_results.iter().map(|r| r.id).collect();
    let hnsw_ids: std::collections::HashSet<_> = hnsw_results.iter().map(|r| r.id).collect();
    let intersection = bf_ids.intersection(&hnsw_ids).count();
    let recall = intersection as f64 / k as f64;

    println!("📈 Results:");
    println!("   Brute-force: {:?}", bf_search_time);
    println!("   HNSW:        {:?}", hnsw_search_time);
    println!(
        "   Speedup:     {:.1}x",
        bf_search_time.as_nanos() as f64 / hnsw_search_time.as_nanos() as f64
    );
    println!("   Recall@{}:   {:.0}%", k, recall * 100.0);
    println!();

    println!("✨ HNSW provides fast approximate search with high recall!");
}
