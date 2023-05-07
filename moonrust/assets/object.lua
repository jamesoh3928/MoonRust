local myObject = {
  x = 0,
  y = 0,

  move = function(myObject, dx, dy)
    myObject.x = myObject.x + dx
    myObject.y = myObject.y + dy
  end,

  getPosition = function(myObject)
    return myObject.x, myObject.y
  end
}

myObject:move(myObject, 10, 20)
local x, y = myObject:getPosition(myObject)

print("Position:", x, y)