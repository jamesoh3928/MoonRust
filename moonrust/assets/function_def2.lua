hello_world = function ()
  print("Hello, World!")
end
hello_world()

function do_nothing()
end
print("hello")
print(do_nothing())
print("world")

swap = function (a, b)
  return b, a
end
x, y = swap(1, 2)
print(x, y)

function outer_function(x)
  function inner_function(y)
    return x + y
  end
  return inner_function
end
print(outer_function(1)(2))