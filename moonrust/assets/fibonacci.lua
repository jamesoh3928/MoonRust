-- This program prints the Fibonacci sequence.

-- Create a function to generate the Fibonacci sequence.
function fibonacci(n)
    if n == 0 then
      return 0
    elseif n == 1 then
      return 1
    else
      return fibonacci(n - 1) + fibonacci(n - 2)
    end
  end
  
  -- Prompt the user for a number.
  io.write("Enter a number: ")
  local n = tonumber(io.read())
  
  -- Print the Fibonacci sequence up to the given number.
  for i = 0, n do
    io.write(fibonacci(i) .. " ")
  end
  io.write("\n")
  