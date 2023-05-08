function game_of_life(board)
  local new_board = {}
  for i = 1, #board do
    row = board[i]
    new_board[i] = {}
    for j = 1, #board[i] do
      cell = row[j]
      local alive_neighbors = 0
      for di = -1, 1 do
        for dj = -1, 1 do
          if di ~= 0 or dj ~= 0 then
            local ni = i + di
            local nj = j + dj
            if ni >= 1 and ni <= #board and nj >= 1 and nj <= #board[ni] then
              alive_neighbors = alive_neighbors + board[ni][nj]
            end
          end
        end
      end

      if cell == 1 then
        if alive_neighbors < 2 or alive_neighbors > 3 then
          new_board[i][j] = 1
        else
          new_board[i][j] = 0
        end
      else
        if alive_neighbors == 3 then
          new_board[i][j] = 1
        else
          new_board[i][j] = 0
        end
      end
    end
  end

  return new_board
end

function print_board(board)
  local row = ""
  local border = ""
  for i = 1, #board*2+1 do
    border = border.."-"
  end
  print(border)
  for i = 1, #board do
    row = "|"
    for j = 1, #board[i] do
      if board[i][j] == 1 then
        row = row.."*"
      else
        row = row.." "
      end
      row = row.."|"
    end
    print(row)
    print(border)
  end
end

local board = {}
board[1] = {1, 0, 1, 1}
board[2] = {0, 0, 0, 1}
board[3] = {1, 1, 1, 1}
board[4] = {1, 1, 1, 1}

print("Enter the numbe of rounds: ")
local rounds = read("*number")
print("Round 0:")
print_board(board)
for i = 1, rounds do
  print("Round "..i..":")
  board = game_of_life(board)
  print_board(board)
  print("")
end
