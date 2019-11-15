"""
As it turns out, you got the Elves' plan backwards. They actually want to know
how many recipes appear on the scoreboard to the left of the first recipes
whose scores are the digits from your puzzle input.

51589 first appears after 9 recipes.
01245 first appears after 5 recipes.
92510 first appears after 18 recipes.
59414 first appears after 2018 recipes.

How many recipes appear on the scoreboard to the left of the score sequence in
your puzzle input?
"""
from common import gen_scores

# This one should be run with PyPy3.5 if you want a sub-15 second answer
# Regular Python needs >20 seconds to do this

if __name__ == "__main__":
    with open('input.txt') as score_sequence:
        score_sequence =\
            list(map(int, score_sequence.read().strip()))

        scores = []

        for n in gen_scores(2, [3, 7]):
            scores.append(n)

            if scores[-len(score_sequence):] == score_sequence:
                print(len(scores) - len(score_sequence))
                break
