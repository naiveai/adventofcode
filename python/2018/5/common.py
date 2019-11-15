import itertools


def react_polymer(polymer_str):
    def should_units_react(p1, p2):
        if not p1.lower() == p2.lower():
            return False

        are_opposite_case =\
            ((p1.islower() and p2.isupper()) or
             (p1.isupper() and p2.islower()))

        return are_opposite_case

    reacted = ""
    for c in polymer_str:
        # If this character reacts with the last
        # one, the last one shouldn't be there.
        if (reacted and should_units_react(c, reacted[-1])):
            reacted = reacted[:-1]
        else:
            reacted += c
    return reacted
