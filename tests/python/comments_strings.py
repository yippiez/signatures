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
