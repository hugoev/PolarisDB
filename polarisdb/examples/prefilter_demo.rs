//! Pre-filtering benchmark - comparing post-filter vs pre-filter search.
//!
//! This example demonstrates how BitmapIndex enables faster filtered search
//! by using roaring bitmaps to narrow down candidates before HNSW traversal.

use polarisdb::prelude::*;
use std::time::Instant;

fn main() {
    println!("🔍 Pre-filter vs Post-filter Benchmark\n");

    // Parameters
    let num_vectors = 10_000;
    let dimension = 64;
    let k = 10;

    println!(
        "📊 Setup: {} vectors, {} dimensions",
        num_vectors, dimension
    );
    println!();

    // Generate vectors with random categories
    println!("⏳ Generating {} vectors with payloads...", num_vectors);
    let categories = ["electronics", "books", "clothing", "sports", "home"];

    let mut hnsw = HnswIndex::new(
        DistanceMetric::Euclidean,
        dimension,
        HnswConfig {
            m: 16,
            m_max0: 32,
            ef_construction: 100,
            ef_search: 50,
        },
    );
    let mut bitmap = BitmapIndex::new();

    for i in 0..num_vectors {
        let v: Vec<f32> = (0..dimension)
            .map(|j| ((i * dimension + j) as f32).sin())
            .collect();
        let category = categories[i % categories.len()];
        let payload = Payload::new().with_field("category", category);

        hnsw.insert(i as u64, v, payload.clone()).unwrap();
        bitmap.insert(i as u64, &payload);
    }
    println!("   ✅ Index built\n");

    // Query
    let query: Vec<f32> = (0..dimension).map(|j| (j as f32 * 0.1).cos()).collect();
    let filter = Filter::field("category").eq("electronics");

    // Warm-up
    let _ = hnsw.search(&query, k, Some(100), Some(filter.clone()));
    let _ = hnsw.search_with_bitmap(&query, k, Some(100), &bitmap.query(&filter));

    // Post-filter search (evaluate filter on each candidate)
    println!("🔍 Post-filter search...");
    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let _ = hnsw.search(&query, k, Some(100), Some(filter.clone()));
    }
    let post_filter_time = start.elapsed() / iterations;

    // Pre-filter search (use bitmap to filter first)
    println!("🔍 Pre-filter search...");
    let start = Instant::now();
    let valid_ids = bitmap.query(&filter); // This is fast - O(1) set operations
    for _ in 0..iterations {
        let _ = hnsw.search_with_bitmap(&query, k, Some(100), &valid_ids);
    }
    let pre_filter_time = start.elapsed() / iterations;

    println!();
    println!("📈 Results (averaged over {} iterations):", iterations);
    println!("   Post-filter: {:?}", post_filter_time);
    println!("   Pre-filter:  {:?}", pre_filter_time);

    let speedup = post_filter_time.as_nanos() as f64 / pre_filter_time.as_nanos() as f64;
    if speedup > 1.0 {
        println!("   Speedup:     {:.2}x faster with pre-filter", speedup);
    } else {
        println!("   Note: Pre-filter has overhead for small result sets");
    }

    // Show filter selectivity
    println!();
    println!("📊 Filter stats:");
    println!("   Total vectors:    {}", num_vectors);
    println!(
        "   Matching filter:  {} ({:.1}%)",
        valid_ids.len(),
        valid_ids.len() as f64 / num_vectors as f64 * 100.0
    );
    println!();

    println!("✨ Pre-filtering with BitmapIndex enables efficient metadata queries!");
}
