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
