"""Minimal ASGI example that uses fastjson."""

from __future__ import annotations

from starlette.applications import Starlette
from starlette.responses import JSONResponse
from starlette.routing import Route

import fastjson


def homepage(_request):
    data = fastjson.loads(fastjson.dumps({"message": "hello"}))
    return JSONResponse({"echo": data})


app = Starlette(debug=True, routes=[Route("/", homepage)])
