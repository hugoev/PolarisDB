# Installation

## Python

Install from PyPI:

```bash
pip install polarisdb
```

### Requirements

- Python 3.8+
- No native dependencies (pre-built wheels for Linux, macOS, Windows)

### Optional: LangChain Integration

For LangChain support, also install:

```bash
pip install langchain-core langchain-openai
```

## Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
polarisdb = "0.1"
```

### Features

Enable optional features:

```toml
[dependencies]
polarisdb = { version = "0.1", features = ["async"] }
```

| Feature | Description |
|---------|-------------|
| `async` | Tokio-based async API (`AsyncCollection`) |

## Docker

Run the HTTP server:

```bash
docker pull hugoev/polarisdb:latest
docker run -p 8080:8080 -v ./data:/data hugoev/polarisdb
```

## Building from Source

```bash
git clone https://github.com/hugoev/polarisdb.git
cd polarisdb
cargo build --release
```

### Python Bindings from Source

```bash
cd py
pip install maturin
maturin develop --release
```
