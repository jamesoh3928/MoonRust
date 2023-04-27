local function noOp()
end
print(noOp())

local function createCounter()
  local count = 0

  local function increment()
    count = count + 1
  end

  return increment
end
local counter = createCounter()
print(counter() == nil)
print(counter() == nil)
print(counter() == nil)

do
  local x = 1
  function f()
    return x
  end
end
print(f())