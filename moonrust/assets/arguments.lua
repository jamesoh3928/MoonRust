function f()
    return 1, 2, 3
end

function t(a, b, c, d, e, f)
    return a, b, c, d, e, f
end

print(t(f(), f()))

a, b, c = f()
print(a)