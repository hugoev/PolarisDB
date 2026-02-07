# Contributing

See [CONTRIBUTING.md](https://github.com/hugoev/polarisdb/blob/main/CONTRIBUTING.md) for full guidelines.

## Quick Start

```bash
# Clone
git clone https://github.com/hugoev/polarisdb.git
cd polarisdb

# Build
cargo build

# Test
cargo test --workspace --all-features

# Lint
cargo clippy -- -D warnings
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and clippy
5. Submit PR

## Code Style

- Follow Rust conventions
- Document public APIs with rustdoc
- Add tests for new features
