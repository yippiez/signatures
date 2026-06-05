# Malformed-but-parseable Ruby: syntax that a real Ruby parser would reject
# but the heuristic line-scanner can still handle without panicking.

# Constant with an unusual value expression (no right-hand side after =).
# The extractor should still emit "WEIRD_CONST = ..."
WEIRD_CONST = # value omitted intentionally

# Class declaration with no body (missing `end`).
class Orphan
  ORPHAN_LIMIT = 10

  def initialize
    @x = ORPHAN_LIMIT
  end

  # Method with an extra closing parenthesis — bracket depth goes negative
  # but the extractor should not panic.
  def too_many_close(x))
    x
  end

# Extra `end` with no matching opener — the stack will just pop nothing.
end
end
end

# Module reopened immediately.
module Reprise
  FIRST = 1
end

module Reprise
  SECOND = 2

  def self.sum
    FIRST + SECOND
  end
end

# def with no name (the extractor emits "def" with whatever follows).
def
  # empty
end

# Nested class without closing end — scanner continues gracefully.
class Outer
  class Inner
    DEEP = 99

    def deep_method(a, b, c)
      a + b + c
    end
  # Inner's `end` is missing here

  def outer_method
    42
  end
end

# Trailing content after __END__
__END__
class ShouldNotAppear
  def invisible
  end
end
