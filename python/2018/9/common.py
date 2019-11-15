import itertools
import re
from collections import Counter, UserList, defaultdict, deque


def parse_marble_info(marble_info_str):
    parts_regex = re.compile(r"(?P<num_players>\d+) players; last marble" +
                             r" is worth (?P<last_marble_points>\d+) points")

    result = parts_regex.match(marble_info_str)

    if not result:
        return ValueError(
            f"String {marble_info_str} doesn't look like a marble info string")

    return map(int, result.groups())


class CircularList(UserList):
    def insert(self, index, x):
        self.data.insert(self.wrap_index(index), x)

    def __getitem__(self, index):
        return self.data[self.wrap_index(index)]

    def __setitem__(self, index, x):
        self.data[self.wrap_index(index)] = x

    def wrap_index(self, index):
        return index % len(self.data)


# My solution implements the CircularList above, but in fact, as you'll see in
# the faster solution below, there's already a highly optimized data structure
# for this. This works for Part 1 but is way too slow for part 2. But I'm proud
# that I got the logic right.
def get_winning_player_my_slow(num_players, last_marble_points):
    player_points_counter = Counter()

    marbles_on_board = CircularList([0])
    current_marble_index = 0

    marbles_remaining = list(range(1, last_marble_points + 1))

    for player_num in itertools.cycle(range(1, num_players + 1)):
        if not marbles_remaining:
            break

        marble_to_place = min(marbles_remaining)

        if marble_to_place % 23 != 0:
            marbles_on_board.insert(current_marble_index + 2, marble_to_place)
            current_marble_index = marbles_on_board.index(marble_to_place)
        else:
            seven_marbles_counterclock_index =\
                marbles_on_board.wrap_index(current_marble_index - 7)

            seven_marbles_counterclock =\
                marbles_on_board.pop(seven_marbles_counterclock_index)

            player_points_counter[player_num] +=\
                marble_to_place + seven_marbles_counterclock

            current_marble_index = seven_marbles_counterclock_index

        marbles_remaining.remove(marble_to_place)

    return player_points_counter.most_common(1)[0]


# This much faster solution, by Marcus Andrews, uses collections.deque, which
# is implemented in C code. It's a lot faster and has methods for rotating the
# list as if it were circular, which works for this excellently. Additionally,
# lists have an worst-case O(n) cost for insert and append, so by using the
# rotate methods to possition the end of the list correctly, we can use the
# O(1) deque.append operation.
def get_winning_player(num_players, last_marble_points):
    scores = Counter()
    circle = deque([0])

    for marble in range(1, last_marble_points + 1):
        if marble % 23 == 0:
            circle.rotate(7)
            # This is also very clever, I didn't realize that, though it seems
            # obvious in hindsight, a player X will place a marble Y if and
            # only if Y % num_players == X. That also caused slowdown, since
            # I calculated the min() of the marbles remaining every time.
            scores[marble % num_players] += marble + circle.pop()
            circle.rotate(-1)
        else:
            circle.rotate(-1)
            circle.append(marble)

    return scores.most_common(1)[0]
