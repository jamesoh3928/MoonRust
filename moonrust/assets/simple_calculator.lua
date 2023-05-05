-- This program implements a simple calculator.

-- Prompt the user for an operation.
io.write("Enter an operation (+, -, *, /): ")
local operation = io.read()

-- Prompt the user for two numbers.
io.write("Enter two numbers: ")
local number1 = tonumber(io.read())
local number2 = tonumber(io.read())

-- Calculate the result of the operation.
if operation == "+" then
  result = number1 + number2
elseif operation == "-" then
  result = number1 - number2
elseif operation == "*" then
  result = number1 * number2
elseif operation == "/" then
  result = number1 / number2
else
  io.write("Invalid operation.\n")
  return
end

-- Print the result.
io.write("The result is " .. result .. "\n")
