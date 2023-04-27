local x = nil
if x then
  print("x is not nil")
else
  print("x is nil")
end

local x = false
if x then
  print("x is true")
else
  print("x is false")
end

local x = "hello"
if x then
  print("x is truthy")
else
  print("x is falsy")
end

local x = 10
if x < 5 then
  print("x is less than 5")
elseif x < 10 then
  print("x is between 5 and 9")
else
  print("x is greater than or equal to 10")
end

local x = 5
if x < 10 then
  if x > 0 then
    print("x is between 0 and 10")
  else
    print("x is less than or equal to 0")
  end
else
  print("x is greater than or equal to 10")
end
