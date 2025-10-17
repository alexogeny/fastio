from __future__ import annotations

import json

import fastjson


def test_round_trip_matches_stdlib() -> None:
    payload = {"numbers": [1, 2, 3], "name": "fastjson", "nested": {"ok": True}}
    encoded = fastjson.dumps(payload)
    assert json.loads(encoded) == payload
    assert fastjson.loads(encoded) == payload
