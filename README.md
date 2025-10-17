# FastIO

FastIO is a workspace that collects five PyO3-based crates and matching Python packages:

- `fastjson`
- `fastmatch`
- `fastcomp`
- `fastform`
- `fastrate`

Each crate compiles to a CPython extension module using `maturin`, and each Python package re-exports the Rust functions. The goal of this change set is to provide the initial scaffold and a passing test for `fastjson`.

## Repository layout

```
fastio/
├── Cargo.toml
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
├── examples/
└── tests/
```

## Local development

From the repository root:

```bash
python -m venv .venv
source .venv/bin/activate
pip install maturin pytest
cd python/fastjson
maturin develop
cd ../..
pytest -q
```

## Continuous integration

The GitHub Actions workflow in `.github/workflows/build.yml` builds wheels for `fastjson` across Python 3.9–3.13 on Linux, macOS, and Windows, runs the unit tests, and produces an sdist artifact.

## License

This project is dual licensed under MIT or Apache-2.0.
