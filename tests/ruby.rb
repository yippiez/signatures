@@CASE@@ comments_strings
# This file verifies that def/class/module tokens inside comments and
# string/heredoc literals are NOT treated as declarations.

REAL_CONST = 99

# def fake_in_comment(x)   -- should be ignored
# class FakeInComment       -- should be ignored
# module FakeModComment     -- should be ignored

=begin
Everything between =begin and =end is a block comment.

def should_be_ignored(a, b)
  a + b
end

class AlsoIgnored
  NOPE = 1
end

module IgnoredModule
end
=end

class StringTricks
  # Strings containing keywords must be ignored.
  FAKE_DEF    = "def not_a_method; end"
  FAKE_CLASS  = 'class NotAClass; end'
  FAKE_MODULE = "module NotAModule; end"

  def real_method
    msg = "this string contains def fake and class Fake tokens"
    other = 'module M; def inner; end; end'
    msg + other
  end

  def method_with_escape
    s = "she said \"def escaped\" and left"
    s
  end

  def heredoc_example
    # Heredoc body is on subsequent lines; the extractor does not model
    # heredoc terminators so we keep the body free of keyword tokens.
    text = <<~MSG
      Hello, world! No keywords here.
    MSG
    text
  end

  # Double-quoted strings on the same line are stripped before scanning.
  def percent_strings
    a = "def fake_in_dq_string"
    b = 'class FakeInSqString'
    [a, b]
  end

  TRAILING_COMMENT_CONST = 7  # def trailing; end -- ignored
end

def real_top_level
  # def nested_fake  -- ignored
  42
end
@@CASE@@ edge
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
@@CASE@@ malformed
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
@@CASE@@ modules
# Exercises module features: mixins, extend, include, prepend,
# attr_accessor / attr_reader / attr_writer, and module functions.

module Serializable
  VERSION = "0.1"

  def self.included(base)
    base.extend(ClassMethods)
  end

  module ClassMethods
    def from_hash(hash)
      obj = new
      hash.each { |k, v| obj.send(:"#{k}=", v) if obj.respond_to?(:"#{k}=") }
      obj
    end

    def attribute_names
      []
    end
  end

  def to_hash
    self.class.attribute_names.each_with_object({}) do |name, h|
      h[name] = send(name)
    end
  end

  def to_json
    pairs = to_hash.map { |k, v| "\"#{k}\":\"#{v}\"" }
    "{#{pairs.join(",")}}"
  end
end

module Validatable
  VALID_EMAIL_RE = /\A[^@]+@[^@]+\z/

  def valid?
    errors.empty?
  end

  def errors
    @errors ||= []
  end

  def validate!
    unless valid?
      raise "Invalid: #{errors.join(', ')}"
    end
    self
  end
end

module Loggable
  LOG_LEVELS = %w[debug info warn error].freeze

  LOG_LEVELS.each do |level|
    define_method(level) do |msg|
      puts "[#{level.upcase}] #{msg}"
    end
  end

  def self.included(base)
    base.instance_variable_set(:@log_prefix, base.name)
  end
end

class Person
  include Serializable
  include Validatable
  include Loggable

  SPECIES = "Homo sapiens"

  attr_accessor :name, :email, :age
  attr_reader   :id
  attr_writer   :role

  def initialize(id, name, email)
    @id    = id
    @name  = name
    @email = email
    @role  = :user
  end

  def self.attribute_names
    [:name, :email, :age]
  end

  def self.create(attrs)
    p = new(attrs[:id], attrs[:name], attrs[:email])
    p.age = attrs[:age]
    p
  end

  def adult?
    @age.to_i >= 18
  end

  def greet
    "Hello, I am #{@name}"
  end

  private

  def validate_email
    errors << "bad email" unless @email.match?(VALID_EMAIL_RE)
  end
end

module MathHelpers
  PI_APPROX = 3.14159

  module_function

  def circle_area(r)
    PI_APPROX * r * r
  end

  def hypotenuse(a, b)
    Math.sqrt(a**2 + b**2)
  end
end
@@CASE@@ nested
# Deeply nested modules and classes, including singleton methods and
# class-level methods defined via def self.

module Outer
  OUTER_VERSION = "1.0"

  module Middle
    MIDDLE_CONST = 42

    class Inner
      INNER_LIMIT = 100

      def initialize(value)
        @value = value
      end

      def compute
        @value * INNER_LIMIT
      end

      def self.build(value)
        new(value)
      end

      class << self
        def registry
          @registry ||= {}
        end

        def register(name, obj)
          registry[name] = obj
        end
      end
    end

    class AnotherInner < Inner
      EXTRA = :extra

      def initialize(value, label)
        super(value)
        @label = label
      end

      def to_s
        "#{@label}:#{compute}"
      end
    end

    def self.create_inner(value)
      Inner.build(value)
    end

    def self.version
      OUTER_VERSION + "-" + MIDDLE_CONST.to_s
    end
  end

  class TopLevel
    TOP_CONST = "top"

    def initialize
      @data = []
    end

    def add(item)
      @data << item
    end

    def all
      @data.dup
    end
  end

  def self.configure
    yield self if block_given?
  end
end

module Standalone
  def self.utility(x, y)
    x + y
  end
end
@@CASE@@ realworld
# A realistic web-application controller / service object.

HTTP_OK      = 200
HTTP_NOT_FOUND = 404
DEFAULT_PAGE_SIZE = 25

module Api
  module V1
    class UsersController
      ALLOWED_PARAMS = %w[name email role].freeze

      attr_accessor :current_user, :logger

      def initialize(repo, logger)
        @repo   = repo
        @logger = logger
      end

      def index(params = {})
        page  = params.fetch(:page, 1).to_i
        limit = params.fetch(:limit, DEFAULT_PAGE_SIZE).to_i
        users = @repo.all(page: page, limit: limit)
        respond_with(HTTP_OK, users.map { |u| serialize(u) })
      end

      def show(id)
        user = @repo.find(id)
        if user.nil?
          respond_with(HTTP_NOT_FOUND, { error: "not found" })
        else
          respond_with(HTTP_OK, serialize(user))
        end
      end

      def create(params)
        attrs = filter_params(params)
        user  = @repo.create(attrs)
        respond_with(HTTP_OK, serialize(user))
      end

      def update(id, params)
        user = @repo.find(id)
        return respond_with(HTTP_NOT_FOUND, {}) unless user

        user.update(filter_params(params))
        respond_with(HTTP_OK, serialize(user))
      end

      def destroy(id)
        @repo.delete(id)
        respond_with(HTTP_OK, {})
      end

      private

      def serialize(user)
        { id: user.id, name: user.name, email: user.email }
      end

      def filter_params(raw)
        raw.select { |k, _| ALLOWED_PARAMS.include?(k.to_s) }
      end

      def respond_with(status, body)
        { status: status, body: body }
      end
    end
  end
end

module Api
  module V1
    class SessionsController
      def create(credentials)
        token = authenticate(credentials[:email], credentials[:password])
        respond_with(HTTP_OK, { token: token })
      end

      def destroy(token)
        revoke(token)
        respond_with(HTTP_OK, {})
      end

      private

      def authenticate(email, password)
        "fake-token-#{email}"
      end

      def revoke(token)
        # nothing
      end

      def respond_with(status, body)
        { status: status, body: body }
      end
    end
  end
end

def health_check
  { status: "ok" }
end
@@CASE@@ sample
MAX = 10

class Greeter
  GREETING = "hi"

  def initialize(name)
    @name = name
  end

  def greet
    "#{GREETING} #{@name}"
  end
end

module Helpers
  def self.run
  end
end

def add(a, b)
  a + b
end
@@CASE@@ unicode
# Ruby allows non-ASCII identifiers (UTF-8). This file exercises that.
# encoding: utf-8

# Top-level constant with a plain ASCII name but a Unicode value.
GREETING_ZH = "你好"
GREETING_JA  = "こんにちは"
GREETING_AR  = "مرحبا"

module Internationalization
  LOCALE_DEFAULT = "en"

  # Class whose name is pure ASCII but whose methods use non-ASCII names.
  class Traductor
    def initialize(locale)
      @locale = locale
    end

    # Ruby 2.0+ allows method names with non-ASCII letters when encoded UTF-8.
    # We also include ordinary ASCII methods for comparison.
    def translate(key)
      lookup(key)
    end

    def lookup(key)
      key.to_s
    end

    def self.supported_locales
      %w[en fr de zh ja ar]
    end
  end

  module Emoji
    # Constants whose values contain emoji.
    THUMBS_UP   = "\u{1F44D}"
    THUMBS_DOWN = "\u{1F44E}"
    PARTY       = "\u{1F389}"

    def self.decorate(text, icon)
      "#{icon} #{text} #{icon}"
    end

    def self.celebrate(msg)
      decorate(msg, PARTY)
    end
  end

  class Formatter
    NBSP = " "  # non-breaking space
    DASH = "—"  # em dash

    def format_title(title, subtitle = nil)
      parts = [title]
      parts << "#{DASH}#{NBSP}#{subtitle}" if subtitle
      parts.join
    end

    def format_list(items)
      items.map { |item| "#{THUMBS_UP}#{NBSP}#{item}" }.join("\n")
    end

    private

    def sanitize(str)
      str.gsub(/\p{C}/, "")
    end
  end
end

# A standalone method with a Unicode string argument default.
def greet(name, greeting = GREETING_ZH)
  "#{greeting}, #{name}!"
end

def noop
end
