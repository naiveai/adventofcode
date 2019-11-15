"""
According to your calculations, the Elves are going to lose badly. Surely, you
won't mess up the timeline too much if you give them just a little advanced
technology, right?

You need to make sure the Elves not only win, but also suffer no losses: even
the death of a single Elf is unacceptable.

However, you can't go too far: larger changes will be more likely to
permanently alter spacetime.

So, you need to find the outcome of the battle in which the Elves have the
lowest integer attack power (at least 4) that allows them to win without a
single death. The Goblins always have an attack power of 3.

In the first summarized example above, the lowest attack power the Elves need
to win without losses is 15:

#######       #######
#.G...#       #..E..#   E(158)
#...EG#       #...E.#   E(14)
#.#.#G#  -->  #.#.#.#
#..G#E#       #...#.#
#.....#       #.....#
#######       #######

Combat ends after 29 full rounds
Elves win with 172 total hit points left
Outcome: 29 * 172 = 4988
In the second example above, the Elves need only 4 attack power:

#######       #######
#E..EG#       #.E.E.#   E(200), E(23)
#.#G.E#       #.#E..#   E(200)
#E.##E#  -->  #E.##E#   E(125), E(200)
#G..#.#       #.E.#.#   E(200)
#..E#.#       #...#.#
#######       #######

Combat ends after 33 full rounds
Elves win with 948 total hit points left
Outcome: 33 * 948 = 31284
In the third example above, the Elves need 15 attack power:

#######       #######
#E.G#.#       #.E.#.#   E(8)
#.#G..#       #.#E..#   E(86)
#G.#.G#  -->  #..#..#
#G..#.#       #...#.#
#...E.#       #.....#
#######       #######

Combat ends after 37 full rounds
Elves win with 94 total hit points left
Outcome: 37 * 94 = 3478
In the fourth example above, the Elves need 12 attack power:

#######       #######
#.E...#       #...E.#   E(14)
#.#..G#       #.#..E#   E(152)
#.###.#  -->  #.###.#
#E#G#G#       #.#.#.#
#...#G#       #...#.#
#######       #######

Combat ends after 39 full rounds
Elves win with 166 total hit points left
Outcome: 39 * 166 = 6474
In the last example above, the lone Elf needs 34 attack power:

#########       #########
#G......#       #.......#
#.E.#...#       #.E.#...#   E(38)
#..##..G#       #..##...#
#...##..#  -->  #...##..#
#...#...#       #...#...#
#.G...G.#       #.......#
#.....G.#       #.......#
#########       #########

Combat ends after 30 full rounds
Elves win with 38 total hit points left
Outcome: 30 * 38 = 1140

After increasing the Elves' attack power until it is just barely enough
for them to win without any Elves dying, what is the outcome of the combat
described in your puzzle input?
"""
import itertools
from copy import deepcopy

from common import CombatGrid, UnitTypes


class ElfDeath(Exception):
    pass


class CombatGridElfDeath(CombatGrid):
    def modify_elf_power(self, value=3):
        for elf in filter(lambda unit: unit.type_ == UnitTypes.ELF,
                          self.units):
            elf.attack_power = value

    def _attempt_attack(self, unit):
        target = super()._attempt_attack(unit)

        if (target and target not in self.units and
                target.type_ == UnitTypes.ELF):
            raise ElfDeath

        return target


if __name__ == "__main__":
    with open('input.txt') as combat_grid_strings:
        combat_grid_original =\
            CombatGridElfDeath.from_combat_grid_strings(
                combat_grid_strings.read().splitlines())

        for elf_attack_power in itertools.count(4):
            try:
                combat_grid = deepcopy(combat_grid_original)
                combat_grid.modify_elf_power(elf_attack_power)

                full_rounds = 0

                while combat_grid.tick():
                    full_rounds += 1

                print(
                    f"Elves win with attack power {elf_attack_power}",
                    end="\n")
                combat_grid.render()

                print("Outcome:",
                      full_rounds * sum(unit.hp for unit in combat_grid.units))

                break
            except ElfDeath:
                continue
