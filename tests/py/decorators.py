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
