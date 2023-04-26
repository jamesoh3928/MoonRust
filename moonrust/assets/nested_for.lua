for i = 1, 3 do
  for j = 1, 6 do
    if i * j >= 9 then
      break;
    end
    print(i, j)
  end
end
print("Loop ended")