from __future__ import annotations

import fastcomp


DATA = b"hello fastcomp"


def test_gzip_roundtrip() -> None:
    compressed = fastcomp.gzip_compress(DATA)
    assert fastcomp.gzip_decompress(compressed) == DATA


def test_brotli_roundtrip() -> None:
    compressed = fastcomp.brotli_compress(DATA)
    assert fastcomp.brotli_decompress(compressed) == DATA


def test_zstd_roundtrip() -> None:
    compressed = fastcomp.zstd_compress(DATA)
    assert fastcomp.zstd_decompress(compressed) == DATA
