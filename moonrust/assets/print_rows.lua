local function print_row(row_number)
  row = ""
  for i = 1, row_number do
    row = row.."*"
  end
  print(row)
end

for i = 1, 5 do
  print_row(i)
end
