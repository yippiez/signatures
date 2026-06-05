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
