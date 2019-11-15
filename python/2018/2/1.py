"""
Late at night, you sneak to the warehouse - who knows what kinds of paradoxes
you could cause if you were discovered - and use your fancy wrist device to
quickly scan every box and produce a list of the likely candidates (your puzzle
input).

To make sure you didn't miss any, you scan the likely candidate boxes again,
counting the number that have an ID containing exactly two of any letter and
then separately counting those with exactly three of any letter. You can
multiply those two counts together to get a rudimentary checksum and compare it
to what your device predicts.

For example, if you see the following box IDs:

abcdef contains no letters that appear exactly two or three times.
bababc contains two a and three b, so it counts for both.
abbcde contains two b, but no letter appears exactly three times.
abcccd contains three c, but no letter appears exactly two times.
aabcdd contains two a and two d, but it only counts once.
abcdee contains two e.
ababab contains three a and three b, but it only counts once.

Of these box IDs, four of them contain a letter which appears exactly twice,
and three of them contain a letter which appears exactly three times.
Multiplying these together produces a checksum of 4 * 3 = 12.

What is the checksum for your list of box IDs?
"""

from collections import Counter


def get_rudimentary_checsum(box_ids):
    """
    Count the number of strings that contain
    exactly 2 characters, and those that contain exactly 3,
    and multiply those counts together.
    """
    # Initialize one counter object in the beginning, so we won't be
    # initializing counters for each iteration
    cnt = Counter()
    num_contains_twos = 0
    num_contains_threes = 0

    for id_ in box_ids:
        cnt.clear()
        cnt.update(id_)

        num_appearences = dict(cnt).values()

        if 2 in num_appearences:
            num_contains_twos += 1

        # Importantly, these don't count for ones that contain 2s, as it has to
        # be *exactly* 2. Thankfully the Counter makes this easy
        if 3 in num_appearences:
            num_contains_threes += 1

    return num_contains_twos * num_contains_threes


if __name__ == "__main__":
    with open('input.txt') as box_ids:
        print(get_rudimentary_checsum(box_ids))
