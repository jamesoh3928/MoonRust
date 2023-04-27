function test_case_4()
  for i=1,5 do
    for j=1,5 do
      if i + j == 5 then
        return i, j
      end
    end
  end
  return -1, -1
end

print(test_case_4())