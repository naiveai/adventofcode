"""
You realize that 20 generations aren't enough. After all, these plants will
need to last another 1500 years to even reach your timeline, not to mention
your future.

After fifty billion (50000000000) generations, what is the sum of the numbers
of all pots which contain a plant?
"""
import itertools

# There is an exploitable pattern in this particular input sequence, after gen
# >= 102. The pattern always goes True three times, False three times, except
# for the last 10 numbers, which are False two times, and True three times.
# The only thing that changes is the whole pattern shifts around. You can tell
# which positions the point starts and ends at deterministically as well.

# This pattern is generated by Advent of Code and seems unique to each person,
# which is why the value seems so arbritary.

if __name__ == "__main__":
    needed_gen = 50000000000
    starting = needed_gen - 34
    ending = needed_gen + 98

    state = dict(
        itertools.takewhile(lambda kv: kv[0] <= ending - 10,
                            ((i + starting, value) for i, value in enumerate(
                                itertools.cycle([True] * 3 + [False] * 3)))))

    state.update(
        dict(
            zip(range(ending - 9, ending + 1),
                ([False] * 2 + [True] * 3) * 2)))

    print(sum(list(k for k, v in state.items() if v)))