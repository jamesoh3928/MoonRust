function main()
    local user_move = ""
    local computer_move = ""
    local user_wins = 0
    local computer_wins = 0
  
    while true do
      print("Enter your move: rock, paper, or scissors")
      user_move = read()
  
      computer_move = random(2)
      if computer_move == 0 then
        computer_move = "rock"
      elseif computer_move == 1 then
        computer_move = "paper"
      else
        computer_move = "scissors"
      end
  
      if user_move == computer_move then
        print("It's a tie!")
      elseif user_move == "rock" and computer_move == "scissors" then
        print("You win!")
        user_wins = user_wins + 1
      elseif user_move == "paper" and computer_move == "rock" then
        print("You win!")
        user_wins = user_wins + 1
      elseif user_move == "scissors" and computer_move == "paper" then
        print("You win!")
        user_wins = user_wins + 1
      else
        print("The computer wins!")
        computer_wins = computer_wins + 1
      end
  
      print("User: " .. user_wins .. " Computer: " .. computer_wins)
  
      if user_wins >= 2 or computer_wins >= 2 then
        break
      end
    end
    if user_wins > computer_wins then
      print("You win the game!")
    else
      print("The computer wins the game!")
    end
  end
  
  main()