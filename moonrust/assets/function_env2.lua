do
  do 
    function f()
      return x
    end
  end
end
local x = 2
print(f())
print(x)

local a = 1
function g() 
  print(a, b)
end
local b = 3
g()
print(a, b)