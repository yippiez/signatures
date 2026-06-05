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
