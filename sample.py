"""A sample module to demonstrate the signatures tool."""

import os

MAX_RETRIES = 5
DEFAULT_NAME: str = "world"
lowercase_value = 42  # not a constant


def greet(name="world"):
    """Say hello. The word def inside this docstring must be ignored."""
    return f"Hello, {name}!"


async def fetch(
    url,
    *,
    timeout=30,
    retries=MAX_RETRIES,
):
    return await _get(url, timeout=timeout)


class Greeter(Base):
    GREETING = "hi"

    def __init__(self, name):
        self.name = name

    async def greet(self) -> str:
        return f"{self.GREETING}, {self.name}"
