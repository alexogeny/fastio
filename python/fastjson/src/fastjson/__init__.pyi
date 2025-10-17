from typing import Any, Union

BytesLike = Union[bytes, bytearray, memoryview]

def loads(data: BytesLike, *, allow_nan: bool = ...) -> Any: ...

def dumps(obj: Any, *, indent: int | None = ..., ensure_ascii: bool = ...) -> bytes: ...
