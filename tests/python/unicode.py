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
