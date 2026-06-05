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
