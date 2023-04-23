-- Define a new object
local myObject = {
    -- Define some variables
    x = 0,
    y = 0,
  
    -- Define some functions
    move = function(self, dx, dy)
      self.x = self.x + dx
      self.y = self.y + dy
    end,
  
    getPosition = function(self)
      return self.x, self.y
    end
  }
  
  -- Call methods on the object
  myObject:move(10, 20)
  local x, y = myObject:getPosition()
  
  -- Output the position
  print("Position: ", x, y)