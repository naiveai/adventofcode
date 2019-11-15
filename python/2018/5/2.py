"""
One of the unit types is causing problems; it's preventing the polymer from
collapsing as much as it should. Your goal is to figure out which unit type is
causing the most problems, remove all instances of it (regardless of polarity),
fully react the remaining polymer, and measure its length.

For example, again using the polymer dabAcCaCBAcCcaDA from above:

- Removing all A/a units produces dbcCCBcCcD. Fully reacting this polymer
produces dbCBcD, which has length 6.
- Removing all B/b units produces daAcCaCAcCcaDA. Fully reacting this polymer
produces daCAcaDA, which has length 8.
- Removing all C/c units produces dabAaBAaDA. Fully reacting this polymer
produces daDA, which has length 4.
- Removing all D/d units produces abAcCaCBAcCcaA. Fully reacting this polymer
produces abCBAc, which has length 6.

In this example, removing all C/c units was best, producing the answer 4.

What is the length of the shortest polymer you can produce by removing all
units of exactly one type and fully reacting the result?
"""
from common import react_polymer


def optimize_polymer(polymer_str):
    units_to_try = set(polymer_str.lower())
    all_polymers = []
    for unit in units_to_try:
        remove_unit_translator =\
            str.maketrans({unit: None, unit.upper(): None})
        polymer_without_unit = polymer_str.translate(remove_unit_translator)

        all_polymers.append(react_polymer(polymer_without_unit))
    return min(all_polymers, key=len)


if __name__ == "__main__":
    with open('input.txt') as polymer_input:
        polymer_str = polymer_input.read().strip()
        print(len(optimize_polymer(polymer_str)))
