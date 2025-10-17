"""fastcomp public API."""

from __future__ import annotations

from ._fastcomp import (
    BrotliCompressor,
    GzipCompressor,
    ZstdCompressor,
    brotli_compress,
    brotli_decompress,
    gzip_compress,
    gzip_decompress,
    zstd_compress,
    zstd_decompress,
)

__all__ = [
    "BrotliCompressor",
    "GzipCompressor",
    "ZstdCompressor",
    "brotli_compress",
    "brotli_decompress",
    "gzip_compress",
    "gzip_decompress",
    "zstd_compress",
    "zstd_decompress",
]
