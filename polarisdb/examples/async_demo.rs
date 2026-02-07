//! Async API demo using AsyncCollection.
//!
//! Run with: cargo run --example async_demo --features async

use polarisdb::prelude::*;
#[cfg(feature = "async")]
use polarisdb::AsyncCollection;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    println!("Async PolarisDB Demo\n");

    // Create temp directory
    let temp_dir = std::env::temp_dir().join("polarisdb_async_demo");
    let _ = std::fs::remove_dir_all(&temp_dir);

    // Open collection asynchronously
    let config = CollectionConfig::new(128, DistanceMetric::Cosine);
    let collection = AsyncCollection::open_or_create(temp_dir.clone(), config)
        .await
        .expect("Failed to create collection");

    println!("Created async collection\n");

    // Insert vectors concurrently
    println!("⏳ Inserting 1000 vectors concurrently...");
    let start = std::time::Instant::now();

    let mut handles = vec![];
    for i in 0..1000 {
        let col = collection.clone();
        let handle = tokio::spawn(async move {
            let vector: Vec<f32> = (0..128).map(|j| ((i * 128 + j) as f32).sin()).collect();
            let payload = Payload::new()
                .with_field("id", i as i64)
                .with_field("category", if i % 2 == 0 { "even" } else { "odd" });
            col.insert(i as u64, vector, payload).await
        });
        handles.push(handle);
    }

    // Wait for all inserts
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    println!("   [OK] Inserted in {:?}", start.elapsed());
    println!("   Total vectors: {}\n", collection.len());

    // Search asynchronously
    println!("Searching...");
    let query: Vec<f32> = (0..128).map(|i| (i as f32 * 0.1).cos()).collect();
    let results = collection.search(&query, 5, None).await;

    println!("   Top 5 results:");
    for (i, r) in results.iter().enumerate() {
        println!("     {}. ID {} (distance: {:.4})", i + 1, r.id, r.distance);
    }

    // Flush and cleanup
    collection.flush().await.unwrap();
    let _ = std::fs::remove_dir_all(&temp_dir);

    println!("\nAsync demo complete!");
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("Run with: cargo run --example async_demo --features async");
}
