function test_case_1()
  for i=1,5 do
    if i == 3 then
      return i
    end
  end
  return -1
end
print(test_case_1())
