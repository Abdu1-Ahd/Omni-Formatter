# ── CASE 1: Basic Lua functions and variables ─────────────────────────────
local   greeting = "Hello, World!"
local   count=0
local max_retries  =  3

-- ── CASE 2: Function definitions ──────────────────────────────────────────
local function add ( a , b )
    return a + b
end

local function greet( name )
  local msg = "Hello, " .. name .. "!"
    print(msg)
  return msg
end

-- ── CASE 3: Tables (objects) ──────────────────────────────────────────────
local user = {
    id=1,
    name="Alice",
    email = "alice@example.com",
    age   = 30,
}

local function User:new(id, name, email)
    local obj = { id=id, name=name, email=email }
    setmetatable(obj, self)
    self.__index = self
    return obj
end

function User:greet()
    return "Hello, I am " .. self.name
end

-- ── CASE 4: Conditionals and loops ────────────────────────────────────────
local function classify(n)
    if n < 0 then
        return "negative"
    elseif n == 0 then
      return "zero"
    elseif n < 10 then
        return "small"
    else
      return "large"
    end
end

for i=1,10 do
    if i % 2 == 0 then
        print(i .. " is even")
    end
end

-- ── CASE 5: Trailing whitespace ───────────────────────────────────────────
local function trailing()   
    local x = 1   
    local y = 2  
    return x + y
end
