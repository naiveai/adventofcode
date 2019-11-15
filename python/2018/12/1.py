"""
After exploring a little, you discover a long tunnel that contains a row of
small pots as far as you can see to your left and right. A few of them contain
plants - someone is trying to grow things in these geothermally-heated caves.

The pots are numbered, with 0 in front of you. To the left, the pots are
numbered -1, -2, -3, and so on; to the right, 1, 2, 3.... Your puzzle input
contains a list of pots from 0 to the right and whether they do (#) or do not
(.) currently contain a plant, the initial state. (No other pots currently
contain plants.) For example, an initial state of #..##.... indicates that pots
0, 3, and 4 currently contain plants.

Your puzzle input also contains some notes you find on a nearby table: someone
has been trying to figure out how these plants spread to nearby pots. Based
on the notes, for each generation of plants, a given pot has or does not have
a plant based on whether that pot (and the two pots on either side of it)
had a plant in the last generation. These are written as LLCRR => N, where L
are pots to the left, C is the current pot being considered, R are the pots
to the right, and N is whether the current pot will have a plant in the next
generation. For example:

A note like ..#.. => . means that a pot that contains a plant but with no
plants within two pots of it will not have a plant in it during the next
generation.

A note like ##.## => . means that an empty pot with two plants on each side of
it will remain empty in the next generation.

A note like .##.# => # means that a pot has a plant in a given generation if,
in the previous generation, there were plants in that pot, the one immediately
to the left, and the one two pots to the right, but not in the ones immediately
to the right and two to the left.

It's not clear what these plants are for, but you're sure it's important, so
you'd like to make sure the current configuration of plants is sustainable by
determining what will happen after 20 generations.

For example, given the following input:

initial state: #..#.#..##......###...###
...## => #
..#.. => #
.#... => #
.#.#. => #
.#.## => #
.##.. => #
.#### => #
#.#.# => #
#.### => #
##.#. => #
##.## => #
###.. => #
###.# => #
####. => #

For brevity, in this example, only the combinations which do produce a plant
are listed. (Your input includes all possible combinations.) Then, the next 20
generations will look like this:

                 1         2         3
       0         0         0         0
 0: ...#..#.#..##......###...###...........
 1: ...#...#....#.....#..#..#..#...........
 2: ...##..##...##....#..#..#..##..........
 3: ..#.#...#..#.#....#..#..#...#..........
 4: ...#.#..#...#.#...#..#..##..##.........
 5: ....#...##...#.#..#..#...#...#.........
 6: ....##.#.#....#...#..##..##..##........
 7: ...#..###.#...##..#...#...#...#........
 8: ...#....##.#.#.#..##..##..##..##.......
 9: ...##..#..#####....#...#...#...#.......
10: ..#.#..#...#.##....##..##..##..##......
11: ...#...##...#.#...#.#...#...#...#......
12: ...##.#.#....#.#...#.#..##..##..##.....
13: ..#..###.#....#.#...#....#...#...#.....
14: ..#....##.#....#.#..##...##..##..##....
15: ..##..#..#.#....#....#..#.#...#...#....
16: .#.#..#...#.#...##...#...#.#..##..##...
17: ..#...##...#.#.#.#...##...#....#...#...
18: ..##.#.#....#####.#.#.#...##...##..##..
19: .#..###.#..#.#.#######.#.#.#..#.#...#..
20: .#....##....#####...#######....#.#..##.

The generation is shown along the left, where 0 is the initial state.
The pot numbers are shown along the top, where 0 labels the center pot,
negative-numbered pots extend to the left, and positive pots extend toward the
right. Remember, the initial state begins at pot 0, which is not the leftmost
pot used in this example.

After one generation, only seven plants remain. The one in pot 0 matched the
rule looking for ..#.., the one in pot 4 matched the rule looking for .#.#.,
pot 9 matched .##.., and so on.

In this example, after 20 generations, the pots shown as # contain plants, the
furthest left of which is pot -2, and the furthest right of which is pot 34.
Adding up all the numbers of plant-containing pots after the 20th generation
produces 325.

After 20 generations, what is the sum of the numbers of all pots which contain
a plant?
"""
import itertools
import re

import more_itertools
from sortedcontainers import SortedDict

from common import render


def parse_point_info(pot_info):
    def parse_pot_list(pots_str):
        return list(c == '#' for c in pots_str)

    initial_pots_regex =\
        re.compile(r"initial state: *(?P<init_pots>[#.]+)")

    change_rules_regex =\
        re.compile(r"(?P<start>[#.]+) *=> *(?P<result>[#.])")

    initial_pots =\
        parse_pot_list(initial_pots_regex.match(pot_info[0]).groups()[0])

    initial_pots = SortedDict(enumerate(initial_pots))

    change_rules = {
        tuple(parse_pot_list(start)): parse_pot_list(result)[0]
        for start, result in
        [change_rules_regex.match(rule).groups() for rule in pot_info[1:]]
    }

    return (initial_pots, change_rules)


def tick(state, change_rules):
    # For this generation, pots 2 before
    # as well as 2 after might be affected
    min_, max_ = min(state), max(state)
    state[min_ - 1], state[min_ - 2] = (False, ) * 2
    state[max_ + 1], state[max_ + 2] = (False, ) * 2

    state_with_surrounding_points =\
        list(more_itertools.stagger(
            state.values(), offsets=tuple(range(-2, 3)),
            longest=True,
            fillvalue=False))[:-1]

    result_state = SortedDict()

    for i, surrounding_points in zip(state.keys(),
                                     state_with_surrounding_points):
        if surrounding_points in change_rules:
            result_state[i] = change_rules[surrounding_points]
        else:
            result_state[i] = False

    return result_state


if __name__ == "__main__":
    with open('input.txt') as pot_info:
        pot_info = filter(None, map(lambda s: s.strip(), pot_info.readlines()))
        initial_state, change_rules = parse_point_info(list(pot_info))

        state = initial_state
        for i in range(20):
            state = tick(state, change_rules)
            print(f"{i}:", end=' ')
            render(state)

        print("Final:", end=' ')
        render(state)

        print(sum(itertools.compress(state.keys(), state.values())))
