local t = {}
t[1] = "x"
t[2] = "y"
t[30] = 23
t[4] = 45
a = t
print(t[1], t[2], t[30], t[4])
print(a[1], a[2], a[30], a[4])
a[1] = "z"
print(t[1], t[2], t[30], t[4])
print(a[1], a[2], a[30], a[4])
