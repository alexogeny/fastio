"""Minimal ASGI example using FastIO packages."""

from __future__ import annotations

import asyncio
from typing import Callable

import fastcomp
import fastform
import fastjson
import fastmatch
import fastrate


router = fastmatch.Router([("GET", "/", 1)])
rate_limiter = fastrate.TokenBucket(100, 50)


async def app(scope, receive, send):  # type: ignore[override]
    assert scope["type"] == "http"
    if not rate_limiter.acquire():
        await send({
            "type": "http.response.start",
            "status": 429,
            "headers": [(b"content-type", b"text/plain")],
        })
        await send({"type": "http.response.body", "body": b"Too Many Requests"})
        return

    route_id, _params = router.lookup(scope["method"], scope.get("headers", {}).get(b"host"), scope["path"])
    if route_id == 1:
        payload = fastjson.dumps({"message": "hello from FastIO"})
        body = fastcomp.gzip_compress(payload)
        await send({
            "type": "http.response.start",
            "status": 200,
            "headers": [(b"content-type", b"application/json"), (b"content-encoding", b"gzip")],
        })
        await send({"type": "http.response.body", "body": body})
