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
