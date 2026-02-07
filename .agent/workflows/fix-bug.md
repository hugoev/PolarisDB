---
description: How to fix a bug in PolarisDB
---

# Bug Fixing Workflow

## 1. Reproduce the Bug

First, create a minimal test case that fails:

```rust
#[test]
fn test_bug_reproduction() {
    // Minimal code that demonstrates the bug
    let mut index = BruteForceIndex::new(DistanceMetric::Cosine, 3);
    // ... setup that triggers the bug
    
    // This should fail before the fix
    assert!(/* expected behavior */);
}
```

## 2. Locate the Issue

Common debugging approaches:

```bash
# Run specific test with output
cargo test test_bug_reproduction -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_bug_reproduction
```

### Key Files by Component

| Component | Primary File | Related Files |
|-----------|--------------|---------------|
| Search | `index/brute_force.rs`, `index/hnsw.rs` | `distance.rs` |
| Filtering | `filter/mod.rs` | `filter/bitmap_index.rs` |
| Persistence | `collection.rs` | `storage/wal.rs`, `storage/data_file.rs` |
| Payloads | `payload.rs` | `filter/mod.rs` |

## 3. Implement the Fix

Make the minimal change to fix the bug:

```rust
// Before (buggy)
fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
    // Bug: didn't handle k=0
    self.find_nearest(query, k)
}

// After (fixed)
fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
    if k == 0 {
        return Vec::new();
    }
    self.find_nearest(query, k)
}
```

## 4. Ensure Regression Test Passes

// turbo
```bash
cargo test test_bug_reproduction
```

## 5. Run Full Test Suite

// turbo
```bash
cargo test --workspace --all-features
```

## 6. Update CHANGELOG

```markdown
## [Unreleased]

### Fixed
- Handle k=0 in search operations (#issue-number)
```

## 7. Commit

```bash
git add -A
git commit -m "fix(search): handle k=0 edge case

Fixes #123"
```
