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
