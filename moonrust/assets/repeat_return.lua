function test_case_3()
  local i = 1
  repeat
    if i == 3 then
      return i
    end
    i = i + 1
  until i > 5
  return -1
end

print(test_case_3())