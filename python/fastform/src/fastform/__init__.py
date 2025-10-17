"""fastform entry points."""

from __future__ import annotations

from ._fastform import Part, parse_multipart, parse_query

__all__ = ["Part", "parse_multipart", "parse_query"]
