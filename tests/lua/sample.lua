MAX = 10

local function add(a, b)
  return a + b
end

local Account = {}

function Account.new(id)
  return setmetatable({ id = id }, Account)
end

function Account:balance()
  return 0
end
