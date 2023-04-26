do
    local t = {}
    t[f(1)] = g
    t[1] = "x"
    t[2] = "y"
    t.x = 1
    t[3] = f(x)
    t[30] = 23
    t[4] = 45
    a = t
end