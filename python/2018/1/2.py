"""
You notice that the device repeats the same frequency change list over and
over. To calibrate the device, you need to find the first frequency it reaches
twice.

For example, using the same list of changes above, the device would loop as
follows:

Current frequency  0, change of +1; resulting frequency  1.
Current frequency  1, change of -2; resulting frequency -1.
Current frequency -1, change of +3; resulting frequency  2.
Current frequency  2, change of +1; resulting frequency  3.
(At this point, the device continues from the start of the list.)
Current frequency  3, change of +1; resulting frequency  4.
Current frequency 4, change of -2; resulting frequency 2, which has already
been seen.

In this example, the first frequency reached twice is 2. Note that your
device might need to repeat its list of frequency changes many times before a
duplicate frequency is found, and that duplicates might be found while in the
middle of processing the list.

What is the first frequency your device reaches twice?
"""
import itertools
from collections import Counter


def get_result_appears_twice(input_changes):
    """
    Apply changes from input_changes to a result
    and see which one appears twice
    """
    result = 0
    results_cnt = Counter({0: 1})

    for change in itertools.cycle(input_changes):
        result += int(change)

        results_cnt[result] += 1

        if results_cnt[result] == 2:
            return result

    return None


if __name__ == "__main__":
    with open('input.txt') as input_changes:
        print(get_result_appears_twice(input_changes))
