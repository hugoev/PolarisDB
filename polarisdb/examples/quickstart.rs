//! Quick start example for PolarisDB.
//!
//! This example demonstrates basic usage of PolarisDB for semantic search.

use polarisdb::prelude::*;

fn main() -> Result<()> {
    println!("🌟 PolarisDB Quick Start Example\n");

    // Create a brute-force index for 8-dimensional vectors
    // (Using small dimension for demonstration; real embeddings are 384-1536 dim)
    let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 8);

    // Sample documents with their "embeddings" (random for demo)
    let documents = vec![
        (
            1,
            "Introduction to Rust programming",
            [0.9, 0.8, 0.1, 0.0, 0.1, 0.0, 0.2, 0.1],
        ),
        (
            2,
            "Advanced Rust patterns and idioms",
            [0.85, 0.9, 0.15, 0.05, 0.1, 0.0, 0.25, 0.15],
        ),
        (
            3,
            "Python for data science",
            [0.1, 0.2, 0.9, 0.85, 0.0, 0.1, 0.0, 0.2],
        ),
        (
            4,
            "Machine learning fundamentals",
            [0.2, 0.1, 0.7, 0.8, 0.6, 0.7, 0.1, 0.3],
        ),
        (
            5,
            "Systems programming with Rust",
            [0.8, 0.7, 0.2, 0.1, 0.15, 0.05, 0.3, 0.2],
        ),
    ];

    // Insert documents into the index
    println!("📥 Inserting {} documents...", documents.len());
    for (id, title, embedding) in &documents {
        let payload = Payload::new()
            .with_field("title", *title)
            .with_field("id", *id as i64);
        index.insert(*id, embedding.to_vec(), payload)?;
    }
    println!("✅ Index contains {} vectors\n", index.len());

    // Search for documents similar to "Rust programming"
    let query = [0.88, 0.85, 0.12, 0.03, 0.12, 0.02, 0.22, 0.12];
    println!("🔍 Searching for documents similar to 'Rust programming'...\n");

    let results = index.search(query, 3, None);

    println!("📊 Top 3 Results:");
    println!("{:-<60}", "");
    for (rank, result) in results.iter().enumerate() {
        let title = result
            .payload
            .as_ref()
            .and_then(|p| p.get_str("title"))
            .unwrap_or("Unknown");
        println!(
            "  {}. [ID: {}] {} (distance: {:.4})",
            rank + 1,
            result.id,
            title,
            result.distance
        );
    }
    println!("{:-<60}\n", "");

    // Demonstrate filtered search
    println!("🔍 Searching with filter (title contains 'Rust')...\n");
    let filter = Filter::field("title").contains("Rust");
    let filtered_results = index.search(query, 10, Some(filter));

    println!("📊 Filtered Results:");
    println!("{:-<60}", "");
    for result in &filtered_results {
        let title = result
            .payload
            .as_ref()
            .and_then(|p| p.get_str("title"))
            .unwrap_or("Unknown");
        println!(
            "  [ID: {}] {} (distance: {:.4})",
            result.id, title, result.distance
        );
    }
    println!("{:-<60}\n", "");

    println!("✨ Done! PolarisDB is working correctly.");
    Ok(())
}
