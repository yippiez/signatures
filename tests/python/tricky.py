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
