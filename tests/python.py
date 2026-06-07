@@CASE@@ annotation_with_equals
MY_CONST: Annotated[int, Field(default=0)] = 0
FIELD: Annotated[str, Field(min_length=1, max_length=100)] = "default"
LITERAL_CONST: Literal["x=1"] = "x=1"
@@CASE@@ async_double_space
async  def bar():
    pass
@@CASE@@ backslash_continuation
def f(a, \
      b):
    pass
@@CASE@@ big_def_truncation
def big(
    p0,
    p1,
    p2,
    p3,
    p4,
    p5,
    p6,
    p7,
    p8,
    p9,
    p10,
    p11,
    p12,
    p13,
    p14,
    p15,
    p16,
    p17,
    p18,
    p19,
    p20,
    p21,
    p22,
    p23,
    p24,
    p25,
    p26,
    p27,
    p28,
    p29,
    p30,
    p31,
    p32,
    p33,
    p34,
    p35,
    p36,
    p37,
    p38,
    p39,
    p40,
    p41,
    p42,
    p43,
    p44,
    p45,
    p46,
    p47,
    p48,
    p49,
    p50,
    p51,
    p52,
    p53,
    p54,
    p55,
    p56,
    p57,
    p58,
    p59,
    p60,
    p61,
    p62,
    p63,
    p64,
    p65,
    p66,
    p67,
    p68,
    p69,
    p70,
    p71,
    p72,
    p73,
    p74,
    p75,
    p76,
    p77,
    p78,
    p79,
    p80,
    p81,
    p82,
    p83,
    p84,
    p85,
    p86,
    p87,
    p88,
    p89,
    p90,
    p91,
    p92,
    p93,
    p94,
    p95,
    p96,
    p97,
    p98,
    p99,
    p100,
    p101,
):
    pass
@@CASE@@ bracket_in_string_default
def f(x="("):
    pass
@@CASE@@ classes
"""Classes, methods, inheritance, and nested classes."""

from abc import ABC


class Animal(ABC):
    LEGS = 4

    def __init__(self, name):
        self.name = name

    def speak(self):
        raise NotImplementedError


class Dog(Animal):
    SOUND = "woof"

    def speak(self):
        return self.SOUND

    class Collar:
        COLOR = "red"

        def size(self):
            return 1


class Empty:
    pass
@@CASE@@ colon_in_return_type
def f() -> "A: B":
    pass
@@CASE@@ comment_in_class_bases
class MyClass(
    # this is a comment
    Base1,  # primary base
    Mixin,  # for extra functionality
):
    pass
@@CASE@@ comment_in_params
def create_user(
    name: str,  # full name
    email: str,  # email address
    age: int = 18,  # default age
) -> User:
    pass
@@CASE@@ comment_with_paren
def f(
    a,  # has a ( inside
    b,
) -> None:
    return a + b
@@CASE@@ comments_strings
"""
This module tests that fake declarations embedded in comments and strings are ignored.

The tool must NOT emit any signature for the following items found in non-code contexts:

In this docstring itself:
    def fake_in_module_docstring(x, y):
    class FakeInModuleDocstring:
    FAKE_MODULE_CONST = "should not appear"
"""

# Real constant that should appear
REAL_CONSTANT: str = "visible"

# def this_is_commented_out(a, b, c):  <- comment, ignore
# class AlsoCommentedOut:              <- comment, ignore
# COMMENTED_CONST = 99                 <- comment, ignore


def real_function_one() -> str:
    """
    Docstring with embedded fake declarations — all must be ignored.

    Example usage (fake, inside docstring)::

        def example_usage():
            pass

        class ExampleClass:
            EXAMPLE_CONST = 1
            def example_method(self):
                pass

    Notes
    -----
    class NotReal:
        NOT_REAL_CONST = True
        def not_real_method(self):
            ...

    def also_not_real(x: int, y: int) -> int:
        return x + y
    """
    return REAL_CONSTANT


def real_function_two(mode: str = "fast") -> None:
    # This comment also contains traps:
    # def trap_in_body_comment():
    # class TrapClassInBodyComment:
    # TRAP_BODY_CONST = 42

    fake_code_string = """
    def not_a_real_def(a, b):
        return a + b

    class NotARealClass:
        NOT_REAL = True
        def not_real(self):
            pass

    INSIDE_STRING_CONST = 100
    """

    single_line_trap = "def single_line_fake(): pass"
    another_trap = 'class SingleLineFakeClass: TRAP = 1'

    raw_trap = r"""
    def raw_string_trap():
        class InsideRaw:
            RAW_CONST = "raw"
"""

    # Inline comment trap: def inline_trap(): pass  # class InlineTrap: INLINE = 1
    return None


ANOTHER_REAL: int = 42  # real constant


class RealClass:
    """A real class — should appear in output."""

    CLASS_CONST: str = "real"

    def __init__(self) -> None:
        self._data = {}

    def real_method(self) -> dict:
        """
        Method docstring with traps:
            def method_trap():
            class MethodTrapClass:
            METHOD_TRAP_CONST = 0
        """
        # def yet_another_trap(): pass
        template = (
            "def template_trap():\n"
            "    class TemplateClass:\n"
            "        T_CONST = True\n"
        )
        return {"template": template}

    @staticmethod
    def static_real() -> bool:
        fake = b"def bytes_trap(): pass"  # bytes literal, not code
        return True


TRAILING_REAL: float = 2.718
# def trailing_commented(): pass
@@CASE@@ constants
"""
Focuses on the boundary between constants (ALL_CAPS names) and non-constants
(lowercase or mixed-case names). Only ALL_CAPS assignments at module or class
scope should be emitted; everything else is ignored.
"""

from typing import Any, Dict, FrozenSet, List, Optional, Set, Tuple

# ----- Module-level: should appear -----
VERSION: str = "3.1.4"
MAX_CONNECTIONS: int = 128
MIN_TIMEOUT: float = 0.5
BUFFER_SIZE: int = 4096
ALLOWED_METHODS: FrozenSet[str] = frozenset({"GET", "POST", "PUT", "DELETE", "PATCH"})
DEFAULT_HEADERS: Dict[str, str] = {
    "Content-Type": "application/json",
    "Accept": "application/json",
}
RETRY_BACKOFF_FACTORS: Tuple[float, ...] = (0.5, 1.0, 2.0, 4.0, 8.0)
EMPTY_SET: Set[int] = set()
SENTINEL: object = object()

# ----- Module-level: should NOT appear (not ALL_CAPS) -----
version_info = (3, 1, 4)          # tuple, lowercase
_private_const = "hidden"          # leading underscore → skip
maxConnections = 128               # camelCase
bufferSize = 4096                  # camelCase
DefaultHeaders: dict = {}          # PascalCase (mixed)
retry_factors = (0.5, 1.0, 2.0)   # snake_case
__dunder__ = None                  # dunder → skip

# ----- Augmented / compound assignments: ALL_CAPS should appear -----
FEATURE_FLAGS: Dict[str, bool] = {}
FEATURE_FLAGS["dark_mode"] = True  # assignment to subscript — not a declaration line

# ----- Walrus / other exprs: tool should not emit these -----
debug = False


class StatusCodes:
    """HTTP status codes as class-level constants."""

    # Should appear
    OK: int = 200
    CREATED: int = 201
    NO_CONTENT: int = 204
    BAD_REQUEST: int = 400
    UNAUTHORIZED: int = 401
    FORBIDDEN: int = 403
    NOT_FOUND: int = 404
    CONFLICT: int = 409
    UNPROCESSABLE_ENTITY: int = 422
    INTERNAL_SERVER_ERROR: int = 500
    SERVICE_UNAVAILABLE: int = 503

    # Should NOT appear
    success_codes: Tuple[int, ...] = (200, 201, 204)
    client_error_start = 400
    _lookup: Dict[int, str] = {}

    @classmethod
    def is_success(cls, code: int) -> bool:
        return 200 <= code < 300

    @classmethod
    def is_client_error(cls, code: int) -> bool:
        return 400 <= code < 500

    @classmethod
    def is_server_error(cls, code: int) -> bool:
        return 500 <= code < 600


class Units:
    """Physical unit conversion constants."""

    # Length
    METERS_PER_FOOT: float = 0.3048
    METERS_PER_INCH: float = 0.0254
    METERS_PER_MILE: float = 1609.344
    METERS_PER_KM: float = 1000.0

    # Mass
    KG_PER_POUND: float = 0.453592
    KG_PER_OUNCE: float = 0.0283495
    KG_PER_TON: float = 907.185

    # Time
    SECONDS_PER_MINUTE: int = 60
    SECONDS_PER_HOUR: int = 3600
    SECONDS_PER_DAY: int = 86400
    SECONDS_PER_WEEK: int = 604800

    # Should NOT appear
    meters_per_yard = 0.9144
    _cached: Optional[Dict] = None

    @staticmethod
    def convert(value: float, from_unit: str, to_unit: str) -> float:
        raise NotImplementedError


class _PrivateConfig:
    """Private class — constants still appear; the class name itself still appears."""

    TIMEOUT: int = 10
    MAX_SIZE: int = 512

    def apply(self) -> Dict[str, Any]:
        return {"timeout": self.TIMEOUT, "max_size": self.MAX_SIZE}


def describe_constant(name: str, value: Any) -> str:
    """Return a human-readable description of a constant."""
    return f"{name} = {value!r} (type: {type(value).__name__})"


def is_constant_name(name: str) -> bool:
    """Return True if name follows the ALL_CAPS convention."""
    return name.isupper() and name.replace("_", "").isalpha()
@@CASE@@ decorators
"""Decorated functions and properties (decorators are not emitted in v1)."""

import functools


@functools.lru_cache(maxsize=None)
def fib(n):
    return n if n < 2 else fib(n - 1) + fib(n - 2)


class Account:
    RATE = 0.05

    @property
    def balance(self):
        return self._balance

    @balance.setter
    def balance(self, value):
        self._balance = value

    @staticmethod
    def fee():
        return 1.0

    @classmethod
    def default(cls):
        return cls()
@@CASE@@ edge
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
@@CASE@@ empty
# only comments and blank lines, no signatures


# nothing to extract here
@@CASE@@ local_const
def my_function():
    LOCAL_CONST = 42
    return LOCAL_CONST
@@CASE@@ multiline
"""Signatures that span several lines, plus type annotations."""

from typing import Dict, List, Optional


def configure(
    host: str,
    port: int = 8080,
    *args,
    timeout: float = 30.0,
    retries: int = 3,
    **kwargs,
) -> Dict[str, int]:
    return {}


async def gather(
    items: List[str],
    limit: Optional[int] = None,
) -> List[str]:
    return items


class Server(
    BaseHandler,
    metaclass=Meta,
):
    HOST: str = "0.0.0.0"

    def run(self) -> None:
        pass
@@CASE@@ nested
"""
Deeply nested classes and functions.
Tests that the signatures tool correctly indents nested members at 2 spaces per level.
"""

from typing import Any, Callable, Optional


# Top-level constant
MAX_DEPTH: int = 10


class Outer:
    """A class that contains deeply nested classes and functions."""

    OUTER_CONST: str = "outer"

    def outer_method(self) -> str:
        return self.OUTER_CONST

    class Middle:
        """A nested class inside Outer."""

        MIDDLE_CONST: int = 2

        def middle_method(self, x: int) -> int:
            return x * self.MIDDLE_CONST

        class Inner:
            """A doubly nested class inside Middle inside Outer."""

            INNER_CONST: float = 3.14

            def inner_method(self) -> float:
                return self.INNER_CONST

            def inner_helper(self, val: Any) -> bool:
                return val is not None

            class DeepestLevel:
                """Three levels deep."""

                DEEPEST: str = "bottom"

                def deepest_method(self) -> str:
                    return self.DEEPEST

        def another_middle_method(self) -> None:
            pass

    def outer_factory(self) -> "Outer.Middle":
        return self.Middle()


class Registry:
    """Demonstrates functions defined inside methods (closures captured as attrs)."""

    REGISTRY_VERSION: int = 1

    def __init__(self) -> None:
        self._handlers: dict = {}

    def register(self, name: str) -> Callable:
        def decorator(fn: Callable) -> Callable:
            self._handlers[name] = fn
            return fn
        return decorator

    def dispatch(self, name: str, *args: Any, **kwargs: Any) -> Any:
        handler = self._handlers.get(name)
        if handler is None:
            raise KeyError(f"No handler registered for {name!r}")
        return handler(*args, **kwargs)

    class HandlerError(Exception):
        """Raised when a handler fails."""

        CODE: int = 500

        def __init__(self, message: str, handler_name: str) -> None:
            super().__init__(message)
            self.handler_name = handler_name

        def to_dict(self) -> dict:
            return {"error": str(self), "handler": self.handler_name, "code": self.CODE}


def make_counter(start: int = 0, step: int = 1) -> Callable[[], int]:
    """Returns a closure that increments a counter."""
    count = start

    def increment() -> int:
        nonlocal count
        count += step
        return count

    return increment


def pipeline(*stages: Callable) -> Callable:
    """Compose multiple callables into a left-to-right pipeline."""

    def run(value: Any) -> Any:
        result = value
        for stage in stages:
            result = stage(result)
        return result

    return run


class TreeNode:
    """Binary tree node — recursion test case."""

    def __init__(self, value: Any, left: Optional["TreeNode"] = None, right: Optional["TreeNode"] = None) -> None:
        self.value = value
        self.left = left
        self.right = right

    def insert(self, value: Any) -> "TreeNode":
        if value < self.value:
            if self.left is None:
                self.left = TreeNode(value)
            else:
                self.left.insert(value)
        else:
            if self.right is None:
                self.right = TreeNode(value)
            else:
                self.right.insert(value)
        return self

    def height(self) -> int:
        left_h = self.left.height() if self.left else 0
        right_h = self.right.height() if self.right else 0
        return 1 + max(left_h, right_h)

    def to_list(self) -> list:
        result = []
        if self.left:
            result.extend(self.left.to_list())
        result.append(self.value)
        if self.right:
            result.extend(self.right.to_list())
        return result

    class Visitor:
        """Strategy object for tree traversal."""

        ORDER_PRE: str = "pre"
        ORDER_IN: str = "in"
        ORDER_POST: str = "post"

        def visit(self, node: "TreeNode", order: str = "in") -> list:
            if order == self.ORDER_PRE:
                return self._pre(node)
            elif order == self.ORDER_IN:
                return self._in(node)
            return self._post(node)

        def _pre(self, node: Optional["TreeNode"]) -> list:
            if node is None:
                return []
            return [node.value] + self._pre(node.left) + self._pre(node.right)

        def _in(self, node: Optional["TreeNode"]) -> list:
            if node is None:
                return []
            return self._in(node.left) + [node.value] + self._in(node.right)

        def _post(self, node: Optional["TreeNode"]) -> list:
            if node is None:
                return []
            return self._post(node.left) + self._post(node.right) + [node.value]
@@CASE@@ realworld
"""
A realistic web-service-style module: config, models, service layer, CLI entry point.
Mirrors patterns found in production Django/FastAPI codebases.
"""

from __future__ import annotations

import logging
import os
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Sequence, Tuple

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Module-level constants
# ---------------------------------------------------------------------------
DEFAULT_TIMEOUT: int = 30
MAX_RETRIES: int = 3
BASE_URL: str = "https://api.example.com/v2"
SUPPORTED_FORMATS: Tuple[str, ...] = ("json", "msgpack", "protobuf")
_internal_version = "1.0.0-dev"  # lowercase: not a constant


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------


@dataclass
class DatabaseConfig:
    """Database connection settings."""

    HOST: str = "localhost"
    PORT: int = 5432
    NAME: str = "app_db"
    USER: str = "postgres"
    PASSWORD: str = ""
    pool_size: int = 10  # lowercase: not a constant

    def dsn(self) -> str:
        return f"postgresql://{self.USER}:{self.PASSWORD}@{self.HOST}:{self.PORT}/{self.NAME}"

    @classmethod
    def from_env(cls) -> "DatabaseConfig":
        return cls(
            HOST=os.getenv("DB_HOST", "localhost"),
            PORT=int(os.getenv("DB_PORT", "5432")),
            NAME=os.getenv("DB_NAME", "app_db"),
            USER=os.getenv("DB_USER", "postgres"),
            PASSWORD=os.getenv("DB_PASSWORD", ""),
        )


@dataclass
class AppConfig:
    """Top-level application configuration."""

    DEBUG: bool = False
    SECRET_KEY: str = "change-me"
    ALLOWED_HOSTS: List[str] = field(default_factory=list)
    db: DatabaseConfig = field(default_factory=DatabaseConfig)

    def validate(self) -> None:
        if not self.SECRET_KEY or self.SECRET_KEY == "change-me":
            raise ValueError("SECRET_KEY must be set in production")

    def is_production(self) -> bool:
        return not self.DEBUG


# ---------------------------------------------------------------------------
# Domain models
# ---------------------------------------------------------------------------


class User:
    """Represents an authenticated user."""

    ROLES: Tuple[str, ...] = ("viewer", "editor", "admin")

    def __init__(self, user_id: int, username: str, role: str = "viewer") -> None:
        self.user_id = user_id
        self.username = username
        self.role = role
        self._token: Optional[str] = None

    def __repr__(self) -> str:
        return f"User(id={self.user_id}, username={self.username!r})"

    def has_permission(self, action: str) -> bool:
        permissions = {
            "viewer": {"read"},
            "editor": {"read", "write"},
            "admin": {"read", "write", "delete", "manage"},
        }
        return action in permissions.get(self.role, set())

    @property
    def token(self) -> Optional[str]:
        return self._token

    @token.setter
    def token(self, value: str) -> None:
        if not isinstance(value, str) or len(value) < 16:
            raise ValueError("Token must be a string of at least 16 characters")
        self._token = value

    @classmethod
    def anonymous(cls) -> "User":
        return cls(user_id=0, username="anonymous", role="viewer")


class Article:
    """Represents a content article."""

    STATUS_DRAFT: str = "draft"
    STATUS_PUBLISHED: str = "published"
    STATUS_ARCHIVED: str = "archived"

    def __init__(
        self,
        article_id: int,
        title: str,
        body: str,
        author: User,
        tags: Optional[List[str]] = None,
    ) -> None:
        self.article_id = article_id
        self.title = title
        self.body = body
        self.author = author
        self.tags = tags or []
        self.status = self.STATUS_DRAFT

    def publish(self) -> None:
        if self.status != self.STATUS_DRAFT:
            raise RuntimeError("Only draft articles can be published")
        self.status = self.STATUS_PUBLISHED

    def archive(self) -> None:
        self.status = self.STATUS_ARCHIVED

    def word_count(self) -> int:
        return len(self.body.split())

    def summary(self, max_words: int = 50) -> str:
        words = self.body.split()
        return " ".join(words[:max_words]) + ("..." if len(words) > max_words else "")


# ---------------------------------------------------------------------------
# Service layer
# ---------------------------------------------------------------------------


class ArticleService:
    """Business logic for article management."""

    PAGE_SIZE: int = 20

    def __init__(self, config: AppConfig) -> None:
        self._config = config
        self._store: Dict[int, Article] = {}
        self._next_id: int = 1

    def create(self, title: str, body: str, author: User, tags: Optional[List[str]] = None) -> Article:
        article = Article(self._next_id, title, body, author, tags)
        self._store[self._next_id] = article
        self._next_id += 1
        logger.info("Created article %d by %s", article.article_id, author.username)
        return article

    def get(self, article_id: int) -> Article:
        try:
            return self._store[article_id]
        except KeyError:
            raise LookupError(f"Article {article_id} not found")

    def list_published(self, page: int = 1) -> List[Article]:
        published = [a for a in self._store.values() if a.status == Article.STATUS_PUBLISHED]
        start = (page - 1) * self.PAGE_SIZE
        return published[start : start + self.PAGE_SIZE]

    def search(self, query: str, tags: Optional[Sequence[str]] = None) -> List[Article]:
        results = []
        for article in self._store.values():
            if query.lower() in article.title.lower() or query.lower() in article.body.lower():
                if tags is None or any(t in article.tags for t in tags):
                    results.append(article)
        return results

    def delete(self, article_id: int, requestor: User) -> None:
        if not requestor.has_permission("delete"):
            raise PermissionError(f"{requestor.username} cannot delete articles")
        article = self.get(article_id)
        del self._store[article.article_id]


# ---------------------------------------------------------------------------
# CLI helpers
# ---------------------------------------------------------------------------


def parse_args(argv: Optional[List[str]] = None) -> Dict[str, Any]:
    import argparse

    parser = argparse.ArgumentParser(description="Article management CLI")
    parser.add_argument("command", choices=["create", "list", "search", "delete"])
    parser.add_argument("--title", help="Article title")
    parser.add_argument("--body", help="Article body text")
    parser.add_argument("--author-id", type=int, default=1)
    parser.add_argument("--page", type=int, default=1)
    parser.add_argument("--query", help="Search query")
    ns = parser.parse_args(argv)
    return vars(ns)


def setup_logging(debug: bool = False) -> None:
    level = logging.DEBUG if debug else logging.INFO
    logging.basicConfig(level=level, format="%(levelname)s %(name)s: %(message)s")


def main(argv: Optional[List[str]] = None) -> int:
    args = parse_args(argv)
    config = AppConfig(DEBUG=bool(os.getenv("DEBUG")))
    setup_logging(config.DEBUG)
    service = ArticleService(config)
    cmd = args["command"]
    if cmd == "list":
        for a in service.list_published(page=args["page"]):
            print(a)
    elif cmd == "search":
        for a in service.search(args.get("query", "")):
            print(a)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
@@CASE@@ simple
"""Plain top-level functions and constants."""

PI = 3.14159
MAX_USERS = 100
APP_NAME = "demo"
debug = True  # lowercase: not a constant


def add(a, b):
    return a + b


def subtract(a, b):
    return a - b


def main():
    print(add(1, 2))
@@CASE@@ tidy_string_default
def f(x=" ,hello"):
    pass


def g(x=",)"):
    pass
@@CASE@@ tricky
"""Edge cases: docstrings, strings, comments that look like declarations."""

CONFIG_PATH = "/etc/app"
_PRIVATE = "skipped: leading underscore is not treated as a constant"


def real_one():
    """
    This docstring contains traps:
        def fake_function():
        class FakeClass:
        FAKE_CONST = 1
    None of the above should appear.
    """
    code = "def also_not_real(): pass"  # a string, not code
    return code


# def commented_out():  <- a comment, must be ignored
HEADERS = {
    "Accept": "application/json",
    "X-Mode": "test",
}


def with_default_dict(opts={"a": 1, "b": 2}):
    return opts


def lambda_default(key=lambda x: x.id):
    return key
@@CASE@@ triple_class_const
class Real:
    PATTERN = 'prefix"""suffix'
    def method(self):
        pass
@@CASE@@ triple_in_dquote
def a():
    x = "'''"

def b():
    pass
@@CASE@@ triple_in_squote
def a():
    x = '"""'

def b():
    pass
@@CASE@@ unicode
"""
Tests non-ASCII / Unicode identifiers in class names, function names, and constants.
Python 3 allows Unicode letters in identifiers (PEP 3131).
"""

from typing import Any, List

# Constants with ASCII names but Unicode string values
GREETING_EN: str = "Hello"
GREETING_JA: str = "こんにちは"   # "こんにちは"
GREETING_AR: str = "مرحبا"    # "مرحبا"
GREETING_ZH: str = "你好"                       # "你好"
PI_APPROX: float = 3.14159265358979

# Non-ASCII constant names (valid Python 3 identifiers)
ÑOÑO: str = "spanish"
ÜBER_LIMIT: int = 9000
RÉSUMÉ_FIELD: str = "résumé"
ÅNGSTRÖM: float = 1e-10

# Lowercase unicode — should NOT be treated as constants
café = "coffee"
naïve = True
über = "uber"


def greet(name: str) -> str:
    """Return a greeting for a name."""
    return f"{GREETING_EN}, {name}!"


def berechne_summe(zahlen: List[float]) -> float:
    """Compute the sum — German-named function."""
    return sum(zahlen)


def вычислить_среднее(числа: List[float]) -> float:
    """Compute mean — Cyrillic-named function (Russian)."""
    if not числа:
        return 0.0
    return sum(числа) / len(числа)


def حساب_مجموع(أرقام: List[float]) -> float:
    """Arabic-named function: compute sum."""
    return sum(أرقام)


def 合計を計算する(数値リスト: List[float]) -> float:
    """Japanese-named function: compute total."""
    return sum(数値リスト)


class Ünterklasse:
    """Class with a German umlaut name."""

    KLASSE_NAME: str = "Ünterklasse"
    MAXIMALE_GRÖẞE: int = 256

    def __init__(self, wert: Any) -> None:
        self.wert = wert

    def hole_wert(self) -> Any:
        """Return the stored value."""
        return self.wert

    def ist_leer(self) -> bool:
        return self.wert is None


class МатематическийОбъект:
    """Cyrillic class name — MathematicalObject."""

    ВЕРСИЯ: str = "1.0"
    ТОЧНОСТЬ: float = 1e-9

    def __init__(self, значение: float) -> None:
        self.значение = значение

    def абсолютное_значение(self) -> float:
        """Absolute value."""
        return abs(self.значение)

    def является_положительным(self) -> bool:
        return self.значение > 0


class 数学クラス:
    """Japanese class name — MathClass."""

    最大値: int = 2**31 - 1
    最小値: int = -(2**31)

    def __init__(self, 値: float) -> None:
        self.値 = 値

    def 平方根を求める(self) -> float:
        import math
        return math.sqrt(abs(self.値))

    def 切り捨て(self) -> int:
        return int(self.値)


def mixed_αβγ_function(α: float, β: float, γ: float = 0.0) -> float:
    """Mix of ASCII and Greek letters in a function name and parameters."""
    return α * β + γ


def normalize_ñ(text: str) -> str:
    """Normalize text containing ñ."""
    import unicodedata
    return unicodedata.normalize("NFC", text)
@@CASE@@ multiline_triple_quoted_default
def f(
    x="""
    multi
    """,
) -> str:
    return x
@@CASE@@ multiline_string_collection_default
def f(
    items=[
        "a",
        "b",
    ],
) -> list:
    pass
@@CASE@@ raw_two_char_prefix_string
def f(x=rb"ends_with_backslash\"):
    pass

def g() -> int:
    return 1
