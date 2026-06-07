@@CASE@@ comments_strings
-- comments_strings.lua
-- Verifies that declarations inside comments and string literals are ignored.

-- function this_is_a_comment(x) end
-- local function also_comment(y) end
-- MAX_IGNORED = 99

--[[
  Everything inside a block comment should be invisible:
  function block_commented(a, b)
    local function inner_commented(x)
      return x
    end
    return a + b
  end
  BLOCK_CONST = 42
]]

REAL_CONST = 100
LOG_LEVEL = "debug"

local fake1 = "function in_string(x) return x end"
local fake2 = 'local function also_in_string() end'

local long_fake = [[
  function hidden_in_long_string(a, b)
    return a + b
  end
  local function more_hidden()
  end
  HIDDEN_CONST = 7
]]

local eq_fake = [==[
  function hidden_eq(x)
    return x
  end
]==]

function real_function(a, b)
  -- function shadow_inside_body(x) end
  local note = "function not_real(z)"
  return a + b
end

local function another_real(x)
  local s = [[function still_fake(q)]]
  return x * 2
end

-- The comment below contains a plausible table definition — must be ignored:
-- Logger = {}
-- function Logger.log(msg) end

Processor = {}

function Processor.run(input)
  return input
end
@@CASE@@ edge
-- edge.lua
-- Edge cases: non-ASCII identifiers, multi-line signatures, empty params,
-- mixed = vs == guards, and a lightly malformed-but-parseable structure.

MAX_UINT = 0xFFFFFFFF
PI_APPROX = 3.14159265358979

-- Multi-line function signature spanning several lines:
function compute(
  alpha,
  beta,
  gamma
)
  return alpha + beta + gamma
end

-- Empty parameter list:
function noop()
end

-- Vararg:
function log(fmt, ...)
  return string.format(fmt, ...)
end

-- Colon method on a local table (table defined inline without a class header):
local Buf = {}

function Buf:write(data)
  self._data = (self._data or "") .. data
end

function Buf:flush()
  local out = self._data or ""
  self._data = ""
  return out
end

-- This assignment uses == so should NOT be treated as a constant:
local ok = status == MAX_UINT

-- ALL_CAPS on LHS of a plain assignment — must be a constant:
RETRY_DELAY = 500
ERROR_CODE_TIMEOUT = 408

-- A table that starts uppercase but is not a class (lowercase first char after _):
local util = {}

function util.clamp(v, lo, hi)
  if v < lo then return lo end
  if v > hi then return hi end
  return v
end

-- Malformed: missing closing paren — the scanner will gather up to 100 continuation
-- lines then emit whatever it has. Should not panic.
function broken(a, b
  -- body intentionally has no closing paren on the signature line
  return a
end

-- Another real function after the malformed one to confirm recovery:
function after_broken(x)
  return x
end
@@CASE@@ modules
-- modules.lua
-- Module pattern with assigned functions, dot-notation methods, and constants.

local M = {}

API_VERSION = "2"
BASE_URL = "https://api.example.com"
REQUEST_TIMEOUT = 60

Config = {}

function Config.defaults()
  return { timeout = REQUEST_TIMEOUT, retries = 3 }
end

function Config.merge(base, overrides)
  local result = {}
  for k, v in pairs(base) do result[k] = v end
  for k, v in pairs(overrides) do result[k] = v end
  return result
end

M.serialize = function(data)
  if type(data) == "table" then
    local parts = {}
    for k, v in pairs(data) do
      parts[#parts + 1] = tostring(k) .. "=" .. tostring(v)
    end
    return "{" .. table.concat(parts, ",") .. "}"
  end
  return tostring(data)
end

M.deserialize = function(text)
  return text
end

local function validate_key(key)
  return type(key) == "string" and #key > 0
end

function M.set(key, value)
  if not validate_key(key) then
    error("invalid key")
  end
  M._store = M._store or {}
  M._store[key] = value
end

function M.get(key)
  return M._store and M._store[key]
end

return M
@@CASE@@ nested
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
@@CASE@@ oop
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
@@CASE@@ realworld
-- realworld.lua
-- A realistic HTTP router module modelled after common Lua web frameworks.

local M = {}

VERSION = "1.0.0"
DEFAULT_TIMEOUT = 30
MAX_RETRIES = 3

local Router = {}

function Router.new(prefix)
  local r = setmetatable({}, Router)
  r.prefix = prefix or ""
  r.routes = {}
  return r
end

function Router:get(path, handler)
  table.insert(self.routes, { method = "GET", path = path, handler = handler })
end

function Router:post(path, handler)
  table.insert(self.routes, { method = "POST", path = path, handler = handler })
end

function Router:dispatch(req)
  for _, route in ipairs(self.routes) do
    if route.method == req.method and route.path == req.path then
      return route.handler(req)
    end
  end
  return { status = 404, body = "Not Found" }
end

local function build_url(host, path, params)
  local url = host .. path
  if params then
    url = url .. "?" .. M.encode_query(params)
  end
  return url
end

function M.encode_query(params)
  local parts = {}
  for k, v in pairs(params) do
    parts[#parts + 1] = k .. "=" .. tostring(v)
  end
  return table.concat(parts, "&")
end

function M.create_router(prefix)
  return Router.new(prefix)
end

return M
@@CASE@@ sample
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
@@CASE@@ string_constants
NAME = "app"
VERSION = "1.0"
PATH = 'usr/local'
MAX = 10
lower = "ignored"
