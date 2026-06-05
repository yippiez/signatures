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
