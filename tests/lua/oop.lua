-- oop.lua
-- Table-as-class patterns using metatables, dot methods, and colon methods.

Animal = {}
Animal.__index = Animal

function Animal.new(name, sound)
  return setmetatable({ name = name, sound = sound }, Animal)
end

function Animal:speak()
  return self.name .. " says " .. self.sound
end

function Animal:getName()
  return self.name
end

function Animal.from_table(t)
  return Animal.new(t.name, t.sound)
end

Dog = setmetatable({}, { __index = Animal })
Dog.__index = Dog

function Dog.new(name)
  return setmetatable(Animal.new(name, "woof"), Dog)
end

function Dog:fetch(item)
  return self.name .. " fetches " .. item
end

function Dog:sit()
  return self.name .. " sits."
end

local Shape = {}
Shape.__index = Shape

function Shape.new(kind)
  return setmetatable({ kind = kind }, Shape)
end

function Shape:area()
  return 0
end

function Shape:describe()
  return "I am a " .. self.kind
end
