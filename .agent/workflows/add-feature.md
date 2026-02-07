---
description: How to add a new feature to PolarisDB
---

# Adding a New Feature

## Pre-Implementation Checklist

1. [ ] Feature doesn't already exist (search codebase)
2. [ ] Feature fits PolarisDB's scope (embedded vector DB)
3. [ ] API design is consistent with existing patterns

## Implementation Steps

### 1. Design the API

Define types and function signatures in `polarisdb-core/src/`.

```rust
// Example: Adding a new distance metric

// In distance.rs
pub enum DistanceMetric {
    Euclidean,
    Cosine,
    DotProduct,
    Hamming,
    Manhattan,  // NEW
}

impl DistanceMetric {
    pub fn calculate(&self, a: &[f32], b: &[f32]) -> f32 {
        match self {
            // ...existing...
            Self::Manhattan => manhattan_distance(a, b),
        }
    }
}

fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum()
}
```

### 2. Add to Exports

Update `polarisdb-core/src/lib.rs`:

```rust
pub use distance::DistanceMetric;  // Already exported
// If adding new types, add them here
```

### 3. Write Tests

Add unit tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manhattan_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dist = manhattan_distance(&a, &b);
        assert!((dist - 9.0).abs() < 1e-6);
    }
}
```

### 4. Add Documentation

```rust
/// Calculates the Manhattan (L1) distance between two vectors.
///
/// # Formula
///
/// `d(a,b) = Σ|aᵢ - bᵢ|`
///
/// # Example
///
/// ```
/// use polarisdb::DistanceMetric;
/// // Use in index
/// ```
fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    // ...
}
```

### 5. Add Example (if user-facing)

Create `polarisdb/examples/feature_demo.rs`:

```rust
//! Demonstrates the new feature
use polarisdb::prelude::*;

fn main() {
    // Example usage
}
```

### 6. Update CHANGELOG.md

```markdown
## [Unreleased]

### Added
- Manhattan (L1) distance metric
```

## Verification

// turbo
```bash
cargo test --workspace --all-features
```

// turbo
```bash
cargo clippy --workspace --all-features -- -D warnings
```

// turbo
```bash
cargo doc --workspace --no-deps
```
