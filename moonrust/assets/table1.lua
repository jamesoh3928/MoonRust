do
    local t = {}
    t[f(1)] = g
    t[1] = "x"         -- 1st exp
    t[2] = "y"         -- 2nd exp
    t.x = 1            -- t["x"] = 1
    t[3] = f(x)        -- 3rd exp
    t[30] = 23
    t[4] = 45          -- 4th exp
    a = t
end