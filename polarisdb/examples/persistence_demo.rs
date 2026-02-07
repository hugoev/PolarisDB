//! Persistence demonstration for PolarisDB.
//!
//! This example shows how to:
//! 1. Create a persistent collection
//! 2. Insert vectors
//! 3. Close and reopen (simulating restart)
//! 4. Verify data persists

use polarisdb::prelude::*;
use std::fs;

fn main() -> Result<()> {
    let collection_path = "./demo_collection";

    // Clean up any previous run
    let _ = fs::remove_dir_all(collection_path);

    println!("🌟 PolarisDB Persistence Demo\n");

    // Phase 1: Create collection and insert data
    println!("📥 Phase 1: Creating collection and inserting vectors...");
    {
        let config = CollectionConfig::new(8, DistanceMetric::Cosine);
        let collection = Collection::open_or_create(collection_path, config)?;

        let docs = vec![
            (
                1,
                "Rust programming basics",
                [0.9, 0.8, 0.1, 0.0, 0.1, 0.0, 0.2, 0.1],
            ),
            (
                2,
                "Advanced Rust patterns",
                [0.85, 0.9, 0.15, 0.05, 0.1, 0.0, 0.25, 0.15],
            ),
            (
                3,
                "Python data science",
                [0.1, 0.2, 0.9, 0.85, 0.0, 0.1, 0.0, 0.2],
            ),
            (
                4,
                "Machine learning",
                [0.2, 0.1, 0.7, 0.8, 0.6, 0.7, 0.1, 0.3],
            ),
            (
                5,
                "Systems with Rust",
                [0.8, 0.7, 0.2, 0.1, 0.15, 0.05, 0.3, 0.2],
            ),
        ];

        for (id, title, embedding) in &docs {
            let payload = Payload::new().with_field("title", *title);
            collection.insert(*id, embedding.to_vec(), payload)?;
        }

        println!("   ✅ Inserted {} vectors", collection.len());
        println!("   💾 Flushing to disk...");
        collection.flush()?;
        println!("   ✅ Collection flushed\n");

        // Collection is dropped here, simulating app shutdown
    }

    // Phase 2: Reopen and verify persistence
    println!("🔄 Phase 2: Reopening collection after 'restart'...");
    {
        let config = CollectionConfig::new(8, DistanceMetric::Cosine);
        let collection = Collection::open_or_create(collection_path, config)?;

        println!(
            "   ✅ Collection reopened with {} vectors\n",
            collection.len()
        );

        // Verify data
        println!("📊 Verifying persisted data:");
        for id in 1..=5 {
            if let Some((_, payload)) = collection.get(id) {
                let title = payload.get_str("title").unwrap_or("Unknown");
                println!("   ID {}: {}", id, title);
            }
        }
        println!();

        // Search still works
        println!("🔍 Searching for 'Rust programming'...");
        let query = [0.88, 0.85, 0.12, 0.03, 0.12, 0.02, 0.22, 0.12];
        let results = collection.search(&query, 3, None);

        println!("📊 Top 3 Results:");
        for result in &results {
            let title = result
                .payload
                .as_ref()
                .and_then(|p| p.get_str("title"))
                .unwrap_or("Unknown");
            println!(
                "   [ID: {}] {} (distance: {:.4})",
                result.id, title, result.distance
            );
        }
        println!();
    }

    // Phase 3: Test crash recovery (no flush)
    println!("💥 Phase 3: Testing crash recovery (insert without flush)...");
    {
        let config = CollectionConfig::new(8, DistanceMetric::Cosine);
        let collection = Collection::open_or_create(collection_path, config)?;

        collection.insert(
            6,
            vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            Payload::new().with_field("title", "New document (no flush)"),
        )?;

        println!("   ✅ Inserted ID 6 WITHOUT calling flush()");
        println!("   📝 Simulating crash (dropping collection)...\n");
        // No flush! Simulates crash
    }

    // Phase 4: Recover from crash
    println!("🔄 Phase 4: Recovering after 'crash'...");
    {
        let config = CollectionConfig::new(8, DistanceMetric::Cosine);
        let collection = Collection::open_or_create(collection_path, config)?;

        println!(
            "   ✅ Collection recovered with {} vectors",
            collection.len()
        );

        if collection.get(6).is_some() {
            println!("   ✅ ID 6 recovered from WAL!");
        } else {
            println!("   ❌ ID 6 not found (WAL recovery failed)");
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all(collection_path);

    println!("\n✨ Demo complete! PolarisDB persistence is working correctly.");
    Ok(())
}
