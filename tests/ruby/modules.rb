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
