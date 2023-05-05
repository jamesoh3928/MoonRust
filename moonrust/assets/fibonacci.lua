function fibonacci(n)
  if n == 0 then
    return 0
  elseif n == 1 then
    return 1
  else
    return fibonacci(n - 1) + fibonacci(n - 2)
  end
end
  
print("Enter a number: ")
local n = read("*number")
  
for i = 0, n do
  print(fibonacci(i))
end