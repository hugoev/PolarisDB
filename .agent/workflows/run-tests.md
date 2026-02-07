---
description: Testing procedures for PolarisDB
---

# Testing Workflow

## Quick Test Commands

// turbo-all

### Run All Tests
```bash
cargo test --workspace
```

### Run All Tests with Features
```bash
cargo test --workspace --all-features
```

### Run Specific Test
```bash
cargo test test_name
```

### Run Tests with Output
```bash
cargo test --workspace -- --nocapture
```

### Run Only Doc Tests
```bash
cargo test --doc --workspace
```

### Run Only Unit Tests (no doc tests)
```bash
cargo test --lib --workspace
```

## Test Organization

| Type | Location | Command |
|------|----------|---------|
| Unit | `#[cfg(test)]` in source files | `cargo test --lib` |
| Doc | `///` comments with examples | `cargo test --doc` |
| Integration | `tests/` directory | `cargo test --tests` |
| Benchmarks | `benches/` directory | `cargo bench` |

## Writing New Tests

### Unit Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = create_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_error_case() {
        let result = function_that_can_fail(bad_input);
        assert!(result.is_err());
    }
}
```

### Doc Test Pattern

```rust
/// Searches for nearest neighbors.
///
/// # Example
///
/// ```
/// use polarisdb::prelude::*;
///
/// let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
/// index.insert(1, vec![1.0, 0.0, 0.0], Payload::new()).unwrap();
/// let results = index.search(&[1.0, 0.0, 0.0], 10, None);
/// assert_eq!(results.len(), 1);
/// ```
pub fn search(&self, query: &[f32], k: usize, filter: Option<Filter>) -> Vec<SearchResult> {
```

## Coverage Check

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --out Html
```

## Linting

### Clippy (Required)
```bash
cargo clippy --workspace --all-features -- -D warnings
```

### Format Check
```bash
cargo fmt --all -- --check
```

## Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench distance
```

## CI Checks

Before pushing, ensure:

1. `cargo test --workspace --all-features` ✓
2. `cargo clippy --workspace --all-features -- -D warnings` ✓
3. `cargo fmt --all -- --check` ✓
4. `cargo doc --workspace --no-deps` (no warnings) ✓
