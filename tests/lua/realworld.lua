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
