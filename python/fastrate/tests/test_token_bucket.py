from __future__ import annotations

import time
from dataclasses import dataclass

import fastrate


def test_token_bucket_basic() -> None:
    bucket = fastrate.TokenBucket(2, 1, time_source=time.perf_counter)
    assert bucket.acquire()
    assert bucket.acquire()
    assert not bucket.acquire()


@dataclass
class _Clock:
    value: float = 0.0

    def __call__(self) -> float:
        return self.value

    def advance(self, seconds: float) -> None:
        self.value += seconds


def test_token_bucket_fractional_refill() -> None:
    clock = _Clock()
    bucket = fastrate.TokenBucket(3, 3, time_source=clock)

    assert bucket.acquire(3)
    assert not bucket.acquire()

    clock.advance(0.25)
    assert not bucket.acquire()

    clock.advance(0.25)
    assert bucket.acquire()
    assert not bucket.acquire()

    clock.advance(1.0)
    assert bucket.acquire(2)
