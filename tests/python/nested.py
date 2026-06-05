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
