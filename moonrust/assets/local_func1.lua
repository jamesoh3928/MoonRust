local function add(x, y)
  return x + y
end
print(add(1, 2) == 3)
print(add(-5, 10) == 5)

local function concatenateStrings(a, b)
  local separator = " "
  return a .. separator .. b
end
print(concatenateStrings("hello", "world") == "hello world")
print(concatenateStrings("foo", "bar") == "foo bar")
