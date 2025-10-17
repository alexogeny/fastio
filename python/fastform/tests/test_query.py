from __future__ import annotations

import fastform


def test_parse_query_decodes_pairs() -> None:
    result = fastform.parse_query(b"foo=bar&baz=qux")
    assert result == [(b"foo", b"bar"), (b"baz", b"qux")]
