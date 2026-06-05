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
