
function test_case_2()
  local i = 1
  while i <= 5 do
    if i == 3 then
      return i
    end
    i = i + 1
  end
  return -1
end

print(test_case_2())