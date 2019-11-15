"""
What would the new winning Elf's score be if the number of the last marble were
100 times larger?
"""
from common import get_winning_player, parse_marble_info

if __name__ == "__main__":
    with open('input.txt') as marble_info:
        num_players, last_marble_points =\
            parse_marble_info(marble_info.read().strip())

        winning_player = get_winning_player(
            num_players, last_marble_points * 100)

        print(f"Elf #{winning_player[0]} won with {winning_player[1]} points")
