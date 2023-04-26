local function add(a, b)
    return a + b
end

local function subtract(a, b)
    return a - b
end

local function multiply(a, b)
    return a * b
end

local function divide(a, b)
    return a / b
end

print("Enter the first number: ")
local a = tonumber(io.read())

print("Enter the operator (+, -, *, /): ")
local op = io.read()

print("Enter the second number: ")
local b = tonumber(io.read())

if op == "+" then
    print(a .. " + " .. b .. " = " .. add(a, b))
elseif op == "-" then
    print(a .. " - " .. b .. " = " .. subtract(a, b))
elseif op == "*" then
    print(a .. " * " .. b .. " = " .. multiply(a, b))
elseif op == "/" then
    print(a .. " / " .. b .. " = " .. divide(a, b))
else
    print("Invalid operator")
end