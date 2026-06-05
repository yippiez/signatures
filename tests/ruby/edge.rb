# Edge cases: operator methods, endless methods, one-liner defs,
# multiline signatures, __END__ data section, and unusual but valid Ruby.

# Constants at the top level.
ZERO        = 0
MAX_RETRIES = 3
EMPTY_ARY   = [].freeze

class MathOps
  # Operator-method definitions.
  def +(other)
    self.class.new(@val + other.val)
  end

  def -(other)
    self.class.new(@val - other.val)
  end

  def *(other)
    self.class.new(@val * other.val)
  end

  def ==(other)
    @val == other.val
  end

  def <=>(other)
    @val <=> other.val
  end

  def [](index)
    @data[index]
  end

  def initialize(val)
    @val = val
    @data = []
  end

  protected

  def val
    @val
  end
end

class EndlessMethods
  # Ruby 3 endless (one-expression) methods: no `end` keyword.
  def double(x) = x * 2
  def square(x) = x ** 2
  def negate(x) = -x
  def identity(x) = x

  # Predicate variant.
  def zero?(x) = x == 0

  # These are ordinary methods for comparison.
  def triple(x)
    x * 3
  end
end

class MultilineSigs
  # A method whose parameter list spans several lines.
  def long_method(
    first_param,
    second_param,
    third_param,
    fourth_param = nil,
    **opts
  )
    [first_param, second_param, third_param, fourth_param, opts]
  end

  # Keyword arguments spanning multiple lines.
  def configure(
    host:,
    port: 80,
    ssl: false,
    timeout: 30
  )
    { host: host, port: port, ssl: ssl, timeout: timeout }
  end

  def self.factory(
    type,
    options = {}
  )
    new
  end
end

# One-liner defs with semicolons.
def noop; end
def always_true; true; end
def answer; 42; end

# Method with a splat and block parameter.
def forward(*args, &blk)
  blk.call(*args)
end

# Method with double-splat.
def options_only(**opts)
  opts
end

# Everything after __END__ is raw data; the scanner must stop here.
__END__
def this_is_data_not_code
  class AlsoData
  end
end
MODULE NotCode
