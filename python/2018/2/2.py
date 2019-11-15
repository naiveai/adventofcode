"""
The boxes will have IDs which differ by exactly one character at the same
position in both strings. For example, given the following box IDs:

abcde
fghij
klmno
pqrst
fguij
axcye
wvxyz

The IDs abcde and axcye are close, but they differ by two characters (the
second and fourth). However, the IDs fghij and fguij differ by exactly one
character, the third (h and u). Those must be the correct boxes.

What letters are common between the two correct box IDs? (In the example above,
this is found by removing the differing character from either ID, producing
fgij.)
"""
import itertools


def differs_by_one_char_same_len(str1, str2):
    """
    Returns the index of the difference if the two strings have a difference in
    exactly 1 place. Must be of the same length. Returns -1 otherwise.
    """
    if len(str1) != len(str2):
        raise ValueError("Strings aren't the same length")

    one_difference_found = False
    found_index = 0
    for i, (chr1, chr2) in enumerate(zip(str1, str2)):
        if chr1 != chr2:
            if one_difference_found:
                return -1
            one_difference_found = True
            found_index = i
    return found_index


def find_pair_differs_by_one_char(box_ids):
    """
    Find a pair of ids that differ by only one character
    """
    for str1, str2 in itertools.combinations(box_ids, r=2):
        difference_result = differs_by_one_char_same_len(str1, str2)
        if difference_result != -1:
            return (difference_result, (str1, str2))
    return None


if __name__ == "__main__":
    with open('input.txt') as box_ids:
        common_index, pair = find_pair_differs_by_one_char(box_ids)
        common_str = pair[0][:common_index] + pair[0][common_index + 1:]
        print(common_str.strip())
