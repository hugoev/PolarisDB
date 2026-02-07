//! Ollama RAG Example - Semantic search with local LLM embeddings
//!
//! This example demonstrates a complete RAG pipeline using:
//! - Ollama for embedding generation (nomic-embed-text model)
//! - PolarisDB for vector storage and search
//!
//! Prerequisites:
//! 1. Install Ollama: https://ollama.ai
//! 2. Pull the embedding model: ollama pull nomic-embed-text
//! 3. Run with: cargo run --example ollama_rag
//!
//! This example uses sync APIs for simplicity.

use polarisdb::{BruteForceIndex, DistanceMetric, Payload};
use serde::{Deserialize, Serialize};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const OLLAMA_URL: &str = "http://localhost:11434";
const EMBED_MODEL: &str = "nomic-embed-text";
const EMBED_DIM: usize = 768;

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f64>,
}

/// Gets embedding from Ollama (blocking).
fn get_embedding(text: &str) -> Result<Vec<f32>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("{}/api/embeddings", OLLAMA_URL))
        .json(&EmbedRequest {
            model: EMBED_MODEL.to_string(),
            prompt: text.to_string(),
        })
        .send()?;

    if !response.status().is_success() {
        return Err(format!("Ollama error: {}", response.status()).into());
    }

    let embed_response: EmbedResponse = response.json()?;
    Ok(embed_response
        .embedding
        .into_iter()
        .map(|x| x as f32)
        .collect())
}

fn main() -> Result<()> {
    println!("🦙 Ollama RAG Example with PolarisDB\n");

    // Sample documents
    let documents = [
        (
            "doc1",
            "Rust is a systems programming language focused on safety and performance.",
        ),
        (
            "doc2",
            "Python is popular for machine learning and data science applications.",
        ),
        (
            "doc3",
            "JavaScript runs in web browsers and powers interactive websites.",
        ),
        (
            "doc4",
            "Vector databases store embeddings for semantic similarity search.",
        ),
        (
            "doc5",
            "PolarisDB is an embedded vector database optimized for local AI.",
        ),
    ];

    // Create index
    let mut index = BruteForceIndex::new(DistanceMetric::Cosine, EMBED_DIM);

    println!("📚 Indexing {} documents...", documents.len());
    for (i, (id, text)) in documents.iter().enumerate() {
        print!("   Embedding '{}'... ", id);

        match get_embedding(text) {
            Ok(embedding) => {
                if embedding.len() != EMBED_DIM {
                    println!("❌ Wrong dimension: {}", embedding.len());
                    continue;
                }
                let payload = Payload::new()
                    .with_field("doc_id", *id)
                    .with_field("text", *text);
                index.insert(i as u64, embedding, payload)?;
                println!("✅");
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                println!("\n⚠️  Make sure Ollama is running: ollama serve");
                println!("⚠️  And the model is pulled: ollama pull {}", EMBED_MODEL);
                return Ok(());
            }
        }
    }

    println!("\n🔍 Semantic Search Demo\n");

    // Example queries
    let queries = vec![
        "What is a good language for building fast software?",
        "How do I build a website?",
        "Tell me about AI databases",
    ];

    for query in queries {
        println!("Query: \"{}\"", query);

        match get_embedding(query) {
            Ok(query_embedding) => {
                let results = index.search(&query_embedding, 2, None);

                for (i, r) in results.iter().enumerate() {
                    if let Some(payload) = &r.payload {
                        let doc_id = payload.get_str("doc_id").unwrap_or("?");
                        let text = payload.get_str("text").unwrap_or("?");
                        println!(
                            "  {}. [{}] {} (score: {:.3})",
                            i + 1,
                            doc_id,
                            text,
                            1.0 - r.distance // Convert distance to similarity
                        );
                    }
                }
                println!();
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
    }

    println!("✨ RAG demo complete!");
    Ok(())
}
