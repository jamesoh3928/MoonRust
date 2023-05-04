function is_prime(n)
  if n == 1 then
    return false
  end
  for i = 2, n / 2, 1 do
    if n % i == 0 then
      return false
    end
  end
  return true
end

print("Enter a number: ")
local n = read("*number")
if is_prime(n) then
  print(n .. " is prime")
else
  print(n .. " is not prime")
end