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
