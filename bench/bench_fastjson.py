"""Micro benchmark harness for fastjson."""

from __future__ import annotations

import json

import pyperf

import fastjson


def main() -> None:
    payload = json.dumps({"numbers": list(range(10))})
    runner = pyperf.Runner()
    runner.bench_func(
        "fastjson roundtrip",
        lambda: fastjson.dumps(fastjson.loads(payload)),
    )


if __name__ == "__main__":
    main()
