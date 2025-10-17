from collections.abc import Iterable, Iterator
from typing import Any, Optional

BytesLike = bytes | bytearray | memoryview

class Part:
    headers: dict[str, bytes]
    filename: Optional[str]
    name: Optional[str]
    data: bytes


def parse_query(data: BytesLike) -> list[tuple[bytes, bytes]]: ...

def parse_multipart(stream: Any, boundary: BytesLike) -> list[Part]: ...
