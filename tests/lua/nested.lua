-- nested.lua
-- Demonstrates deeply nested local functions and closures.

CACHE_SIZE = 256

function make_counter(start)
  local count = start or 0

  local function increment(step)
    count = count + (step or 1)
  end

  local function reset()
    count = 0
  end

  local function get()
    return count
  end

  return { increment = increment, reset = reset, get = get }
end

function make_pipeline(transforms)
  local function apply(value)
    local function run(index, v)
      if index > #transforms then
        return v
      end
      return run(index + 1, transforms[index](v))
    end
    return run(1, value)
  end

  return apply
end

local function memoize(fn)
  local cache = {}

  local function lookup(key)
    return cache[key]
  end

  local function store(key, value)
    cache[key] = value
  end

  return function(x)
    local cached = lookup(x)
    if cached ~= nil then
      return cached
    end
    local result = fn(x)
    store(x, result)
    return result
  end
end
