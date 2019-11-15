"""
Okay, so the facility is big.

How many rooms have a shortest path from your current location that pass
through at least 1000 doors?
"""
from common import parse_rooms_regex

if __name__ == "__main__":
    with open('input.txt') as rooms_regex:
        rooms_regex =\
            rooms_regex.read().strip()[1:-1]

    grid = parse_rooms_regex(rooms_regex)

    print(sum(map(lambda v: v >= 1000, grid.values())))
