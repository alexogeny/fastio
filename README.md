# FastIO

FastIO is a collection of Rust-accelerated Python extensions designed for high-performance ASGI servers. The repository hosts shared Rust crates and Python packages for JSON processing, routing, compression, form parsing, and rate limiting.

## Repository Layout

```
fastio/
├── Cargo.toml
├── rust-toolchain.toml
├── crates/
│   ├── fastjson/
│   ├── fastmatch/
│   ├── fastcomp/
│   ├── fastform/
│   └── fastrate/
├── python/
│   ├── fastjson/
│   ├── fastmatch/
│   ├── fastcomp/
│   ├── fastform/
│   └── fastrate/
├── bench/
├── ci/
└── examples/
```

Each crate exports a PyO3-powered module and is published as an independent Python distribution via `maturin`. The Python packages include light shims, type hints, and documentation. Benchmarks and examples demonstrate typical ASGI integrations.

## Getting Started

Install the Rust toolchain and Python tooling with [`uv`](https://docs.astral.sh/uv/):

```bash
uv python install 3.11
uv tool install maturin
```

Build an extension module in editable mode from a package directory, for example `fastjson`:

```bash
cd python/fastjson
uv pip install -e .
```

Run the test suite with `pytest` and `cargo nextest`, and use `ruff`, `ty`, `cargo fmt`, and `clippy` to maintain code quality.

## License

Dual licensed under either MIT or Apache 2.0.
