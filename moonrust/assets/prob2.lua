local a = 2
function f()
    a = a + 1
end
b = a
f()
print(a)
print(b)