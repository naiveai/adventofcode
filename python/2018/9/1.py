"""
The Elves play this game by taking turns arranging the marbles in a circle
according to very particular rules. The marbles are numbered starting with 0
and increasing by 1 until every marble has a number.

First, the marble numbered 0 is placed in the circle. At this point, while
it contains only a single marble, it is still a circle: the marble is both
clockwise from itself and counter-clockwise from itself. This marble is
designated the current marble.

Then, each Elf takes a turn placing the lowest-numbered remaining marble into
the circle between the marbles that are 1 and 2 marbles clockwise of the
current marble. (When the circle is large enough, this means that there is one
marble between the marble that was just placed and the current marble.) The
marble that was just placed then becomes the current marble.

However, if the marble that is about to be placed has a number which is a
multiple of 23, something entirely different happens. First, the current player
keeps the marble they would have placed, adding it to their score. In addition,
the marble 7 marbles counter-clockwise from the current marble is removed from
the circle and also added to the current player's score. The marble located
immediately clockwise of the marble that was removed becomes the new current
marble.

The goal is to be the player with the highest score after the last marble is
used up. Assuming the example above ends after the marble numbered 25, the
winning score is 23+9=32 (because player 5 kept marble 23 and removed marble 9,
while no other player got any points in this very short example game).

Here are a few examples:

- 10 players; last marble is worth 1618 points: high score is 8317
- 13 players; last marble is worth 7999 points: high score is 146373
- 17 players; last marble is worth 1104 points: high score is 2764
- 21 players; last marble is worth 6111 points: high score is 54718
- 30 players; last marble is worth 5807 points: high score is 37305

What is the winning Elf's score?
"""
from common import get_winning_player, parse_marble_info

if __name__ == "__main__":
    with open('input.txt') as marble_info:
        num_players, last_marble_points =\
            parse_marble_info(marble_info.read().strip())

        winning_player = get_winning_player(num_players, last_marble_points)

        print(f"Elf #{winning_player[0]} won with {winning_player[1]} points")
