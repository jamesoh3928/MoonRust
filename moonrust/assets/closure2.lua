function testClosure()
  local a = 5
  local function addA(x)
    return x + a
  end
  return addA(10)
end
print(testClosure() == 15)

function testClosure2()
  local t = {x = 5}
  local function addX(y)
    return y + t.x
  end
  return addX(10)
end
print(testClosure2() == 15)

function testClosure3()
  local a = 5
  local function addA(x)
    return x + a
  end
  local function subA(x)
    return x - a
  end
  return addA(subA(20))
end
print(testClosure3() == 15)

function testClosure4()
  local a = 5
  local function outer()
    local b = 10
    local function inner(x)
      return x + a + b
    end
    return inner(20)
  end
  return outer()
end
print(testClosure4() == 35)
