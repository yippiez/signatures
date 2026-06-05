"""
Malformed-but-parseable edge cases:
- extremely long argument lists
- unusual but legal syntax (positional-only params, starred in class bases)
- async generators and async context managers
- __dunder__ methods galore
- class with only a pass body
- function with only an ellipsis body
- deeply chained decorators
- type aliases and TypeVar at module scope
- constants with complex right-hand sides
"""

from __future__ import annotations

import sys
from typing import (
    AsyncGenerator,
    ClassVar,
    Generator,
    Generic,
    Iterator,
    Optional,
    Tuple,
    TypeVar,
    overload,
)

# ---- Type variables (not constants — lowercase by convention) ----
T = TypeVar("T")
K = TypeVar("K")
V = TypeVar("V")
T_co = TypeVar("T_co", covariant=True)

# ---- Module constants ----
PLATFORM: str = sys.platform
VERSION_INFO: Tuple[int, ...] = tuple(sys.version_info[:3])
MAX_INT: int = sys.maxsize
IS_64BIT: bool = sys.maxsize > 2**32
EMPTY_BYTES: bytes = b""
NEWLINE: str = "\n"

# ---- Unusual but legal constant: multiline right-hand side ----
LOOKUP_TABLE: dict = {
    0: "zero",
    1: "one",
    2: "two",
    3: "three",
}

# Not a constant (lowercase)
_sentinel = object()


# ---- Positional-only parameters (PEP 570, Python 3.8+) ----
def pos_only_func(x: int, y: int, /, z: int = 0) -> int:
    return x + y + z


# ---- Many parameters ----
def overloaded_config(
    host: str = "localhost",
    port: int = 8080,
    user: Optional[str] = None,
    password: Optional[str] = None,
    database: Optional[str] = None,
    timeout: float = 30.0,
    connect_timeout: float = 10.0,
    max_overflow: int = 10,
    pool_size: int = 5,
    pool_recycle: int = -1,
    pool_pre_ping: bool = False,
    echo: bool = False,
    /,
    *,
    ssl: bool = False,
    ssl_ca_certs: Optional[str] = None,
    charset: str = "utf8mb4",
    **extra,
) -> dict:
    return {}


# ---- Async generator ----
async def async_range(start: int, stop: int, step: int = 1) -> AsyncGenerator[int, None]:
    i = start
    while i < stop:
        yield i
        i += step


# ---- Sync generator ----
def chunked(iterable, size: int) -> Generator[list, None, None]:
    chunk: list = []
    for item in iterable:
        chunk.append(item)
        if len(chunk) >= size:
            yield chunk
            chunk = []
    if chunk:
        yield chunk


# ---- Overloaded function (typing.overload) ----
@overload
def process(value: int) -> str: ...

@overload
def process(value: str) -> int: ...

def process(value):
    if isinstance(value, int):
        return str(value)
    return len(value)


# ---- Class with only a docstring body ----
class AbstractMarker:
    """Marker interface — no methods or constants at all."""


# ---- Class with only pass ----
class EmptyWithPass:
    pass


# ---- Class with only ellipsis ----
class Protocol:
    ...


# ---- Dunder-heavy class ----
class MagicBox(Generic[T]):
    """Implements nearly every dunder method."""

    CAPACITY: ClassVar[int] = 64

    def __init__(self, value: T) -> None:
        self._value = value

    def __repr__(self) -> str:
        return f"MagicBox({self._value!r})"

    def __str__(self) -> str:
        return str(self._value)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, MagicBox):
            return NotImplemented
        return self._value == other._value

    def __hash__(self) -> int:
        return hash(self._value)

    def __bool__(self) -> bool:
        return bool(self._value)

    def __len__(self) -> int:
        return 1

    def __contains__(self, item: object) -> bool:
        return item == self._value

    def __iter__(self) -> Iterator[T]:
        yield self._value

    def __next__(self) -> T:
        raise StopIteration

    def __enter__(self) -> "MagicBox[T]":
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        return False

    def __add__(self, other: "MagicBox[T]") -> "MagicBox":
        return MagicBox((self._value, other._value))

    def __iadd__(self, other: "MagicBox[T]") -> "MagicBox[T]":
        self._value = (self._value, other._value)  # type: ignore[assignment]
        return self

    def __getitem__(self, key: int) -> T:
        if key != 0:
            raise IndexError(key)
        return self._value

    def __setitem__(self, key: int, value: T) -> None:
        if key != 0:
            raise IndexError(key)
        self._value = value

    def __delitem__(self, key: int) -> None:
        raise TypeError("Cannot delete from MagicBox")

    def __call__(self, *args, **kwargs) -> T:
        return self._value

    def __sizeof__(self) -> int:
        return object.__sizeof__(self)

    def __class_getitem__(cls, item):
        return super().__class_getitem__(item)

    @classmethod
    def empty(cls) -> "MagicBox[None]":
        return cls(None)

    @staticmethod
    def of(value: T) -> "MagicBox[T]":
        return MagicBox(value)


# ---- Chained decorators ----
def decorator_a(fn):
    return fn

def decorator_b(arg):
    def wrapper(fn):
        return fn
    return wrapper

def decorator_c(fn):
    return fn


@decorator_a
@decorator_b("hello")
@decorator_c
def triple_decorated(x: int, y: int) -> int:
    return x + y


# ---- Function with only ellipsis body ----
def abstract_stub(name: str, value: object) -> None: ...


# ---- Walrus operator in a function (should not confuse constant detection) ----
def walrus_demo(data: list) -> Optional[int]:
    if (n := len(data)) > 0:
        return n
    return None
