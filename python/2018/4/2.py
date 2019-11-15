"""
Strategy 2: Of all guards, which guard is most frequently asleep on the same
minute?

Date   ID   Minute
            000000000011111111112222222222333333333344444444445555555555
            012345678901234567890123456789012345678901234567890123456789
11-01  #10  .....####################.....#########################.....
11-02  #99  ........................................##########..........
11-03  #10  ........................#####...............................
11-04  #99  ....................................##########..............
11-05  #99  .............................................##########.....

In the example above, Guard #99 spent minute 45 asleep more than any other
guard or minute - three times in total. (In all other cases, any guard spent
any minute asleep at most twice.)

What is the ID of the guard you chose multiplied by the minute you chose? (In
the above example, the answer would be 99 * 45 = 4455.)
"""
from collections import Counter

from common import get_guard_minutes_asleep, parse_event_list


def most_common_minute(guard_minutes_asleep):
    cnt = Counter()
    most_common_minutes = {}

    for guard_id, timings in guard_minutes_asleep.items():
        timings = list(timings.values())

        cnt.clear()
        for set_ in timings:
            cnt.update(set_)

        most_common_minutes[guard_id] = cnt.most_common(1)[0]

    # We end up with a dictionary with guard ids
    # and a tuple as the key with (minute, frequency).
    # We simply have to pick the minute with that appears
    # the most out of those.

    def key_by_minute_frequency(item):
        return item[1][1]

    most_common_guard_asleep =\
        max(most_common_minutes.items(), key=key_by_minute_frequency)

    return (most_common_guard_asleep[0], most_common_guard_asleep[1][0])


if __name__ == "__main__":
    with open('input.txt') as guard_event_strings:
        sorted_guard_events = parse_event_list(guard_event_strings)

        guard_minutes_asleep = get_guard_minutes_asleep(sorted_guard_events)

        guard, minute = most_common_minute(guard_minutes_asleep)

        print(guard * minute)
