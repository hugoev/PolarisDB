# Contributing to PolarisDB

Thank you for your interest in contributing to PolarisDB! 🎉

We welcome contributions of all kinds: bug reports, feature requests, documentation improvements, and code contributions.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and inclusive in all interactions.

## Getting Started

### Reporting Bugs

Before filing a bug report, please:

1. Search [existing issues](https://github.com/hugoev/polarisdb/issues) to avoid duplicates
2. Use the bug report template when creating a new issue
3. Include a minimal reproduction case
4. Specify your Rust version (`rustc --version`) and OS

### Suggesting Features

We love feature suggestions! Please:

1. Check if the feature already exists or is planned
2. Open a discussion or issue describing your use case
3. Be specific about the problem you're trying to solve

## Development Setup

### Prerequisites

- Rust 1.70 or later (we recommend using [rustup](https://rustup.rs/))
- Git

### Clone and Build

```bash
git clone https://github.com/hugoev/polarisdb.git
cd polarisdb
cargo build
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run with all features
cargo test --workspace --all-features

# Run specific test
cargo test test_name
```

### Run Benchmarks

```bash
cargo bench
```

## Making Changes

### Branching Strategy

1. Fork the repository
2. Create a feature branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. Make your changes in small, focused commits
4. Push to your fork and open a Pull Request

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(hnsw): add delete operation with connection repair
fix(filter): handle null values in equality comparison
docs(readme): add async API examples
perf(distance): optimize cosine similarity with SIMD
```

## Pull Request Process

1. **Ensure CI passes**: All tests, clippy, and formatting checks must pass
2. **Update documentation**: Include rustdoc for new public APIs
3. **Add tests**: New features require tests; bug fixes should include regression tests
4. **Write a clear description**: Explain what changes you made and why
5. **Link related issues**: Reference any issues this PR addresses

### PR Checklist

- [ ] Code compiles without warnings (`cargo build`)
- [ ] All tests pass (`cargo test --workspace --all-features`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for notable changes)

## Coding Standards

### Rust Style

We follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

- Use `rustfmt` for formatting (default settings)
- Use `clippy` for linting
- Prefer `Result` over panics for error handling
- Document all public items with rustdoc

### Code Organization

```
polarisdb/
├── polarisdb-core/      # Core library (no async runtime)
│   └── src/
│       ├── collection.rs   # Persistent collection API
│       ├── distance.rs     # Distance metrics
│       ├── filter/         # Filter expressions
│       ├── index/          # Index implementations
│       ├── storage/        # Persistence layer
│       └── ...
└── polarisdb/           # Main crate (re-exports)
    └── examples/        # Usage examples
```

### Error Handling

- Use the `Error` enum in `error.rs` for all errors
- Provide context in error messages
- Document when functions can fail

### Performance

- Benchmark before and after significant changes
- Avoid unnecessary allocations in hot paths
- Use appropriate data structures (HashMap for O(1) lookups, etc.)

## Testing

### Test Categories

1. **Unit tests**: Test individual functions in `#[cfg(test)]` modules
2. **Integration tests**: Test component interactions in `tests/`
3. **Doc tests**: Examples in documentation that are also tests
4. **Benchmarks**: Performance tests in `benches/`

### Writing Good Tests

```rust
#[test]
fn test_search_returns_k_results() {
    // Arrange
    let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
    for i in 0..10 {
        index.insert(i, vec![i as f32; 3], Payload::new()).unwrap();
    }

    // Act
    let results = index.search(&[0.0, 0.0, 0.0], 5, None);

    // Assert
    assert_eq!(results.len(), 5);
}
```

## Documentation

### Rustdoc Guidelines

Every public item should have documentation:

```rust
/// Searches for the k nearest neighbors to a query vector.
///
/// # Arguments
///
/// * `query` - The query vector (must match index dimension)
/// * `k` - Maximum number of results to return
/// * `filter` - Optional filter to apply to results
///
/// # Returns
///
/// A vector of search results sorted by distance (ascending).
///
/// # Example
///
/// ```
/// use polarisdb::prelude::*;
///
/// let mut index = BruteForceIndex::new(DistanceMetric::Euclidean, 3);
/// index.insert(1, vec![1.0, 0.0, 0.0], Payload::new()).unwrap();
///
/// let results = index.search(&[1.0, 0.0, 0.0], 10, None);
/// assert_eq!(results[0].id, 1);
/// ```
pub fn search(&self, query: &[f32], k: usize, filter: Option<Filter>) -> Vec<SearchResult> {
    // ...
}
```

### README Updates

If your changes affect the public API or add features, update the README.

## Questions?

Feel free to:
- Open a [Discussion](https://github.com/hugoev/polarisdb/discussions)
- Ask in the PR or issue comments
- Reach out on [Discord/Slack/etc.] (if applicable)

Thank you for contributing! 💙
