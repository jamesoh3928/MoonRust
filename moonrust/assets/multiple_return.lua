function f()
    return 1, 2, 3
end

function t()
    return f(), f(), f()
end

print(t(), t())