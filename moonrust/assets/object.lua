local myObject = {
    x = 0,
    y = 0,
  
    move = function(self, dx, dy)
      self.x = self.x + dx
      self.y = self.y + dy
    end,
  
    getPosition = function(self)
      return self.x, self.y
    end
  }
  
  myObject:move(10, 20)
  local x, y = myObject:getPosition()
  
  print("Position: ", x, y)