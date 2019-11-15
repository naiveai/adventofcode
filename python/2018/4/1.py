"""
They've been writing down the ID of the one guard on duty that night - the
Elves seem to have decided that one guard was enough for the overnight shift -
as well as when they fall asleep or wake up while at their post (your puzzle
input).

For example, consider the following records, which have already been organized
into chronological order:

[1518-11-01 00:00] Guard #10 begins shift
[1518-11-01 00:05] falls asleep
[1518-11-01 00:25] wakes up
[1518-11-01 00:30] falls asleep
[1518-11-01 00:55] wakes up
[1518-11-01 23:58] Guard #99 begins shift
[1518-11-02 00:40] falls asleep
[1518-11-02 00:50] wakes up
[1518-11-03 00:05] Guard #10 begins shift
[1518-11-03 00:24] falls asleep
[1518-11-03 00:29] wakes up
[1518-11-04 00:02] Guard #99 begins shift
[1518-11-04 00:36] falls asleep
[1518-11-04 00:46] wakes up
[1518-11-05 00:03] Guard #99 begins shift
[1518-11-05 00:45] falls asleep
[1518-11-05 00:55] wakes up

Timestamps are written using year-month-day hour:minute format. The guard
falling asleep or waking up is always the one whose shift most recently
started. Because all asleep/awake times are during the midnight hour (00:00 -
00:59), only the minute portion (00 - 59) is relevant for those events.

Visually, these records show that the guard_timings are asleep at these times:

Date   ID   Minute
            000000000011111111112222222222333333333344444444445555555555
            012345678901234567890123456789012345678901234567890123456789
11-01  #10  .....####################.....#########################.....
11-02  #99  ........................................##########..........
11-03  #10  ........................#####...............................
11-04  #99  ....................................##########..............
11-05  #99  .............................................##########.....

The columns are Date, which shows the month-day portion of the relevant day;
ID, which shows the guard on duty that day; and Minute, which shows the minutes
during which the guard was asleep within the midnight hour. (The Minute
column's header shows the minute's ten's digit in the first row and the one's
digit in the second row.) Awake is shown as ., and asleep is shown as #.

Note that guard_timings count as asleep on the minute they fall asleep, and
they count as awake on the minute they wake up. For example, because Guard #10
wakes up at 00:25 on 1518-11-01, minute 25 is marked as awake.

If you can figure out the guard most likely to be asleep at a specific time,
you might be able to trick that guard into working tonight so you can have
the best chance of sneaking in. You have two strategies for choosing the best
guard/minute combination.

Strategy 1: Find the guard that has the most minutes asleep. What minute does
that guard spend asleep the most?

In the example above, Guard #10 spent the most minutes asleep, a total of
50 minutes (20+25+5), while Guard #99 only slept for a total of 30 minutes
(10+10+10). Guard #10 was asleep most during minute 24 (on two days, whereas
any other minute the guard was asleep was only seen on one day).

While this example listed the entries in chronological order, your entries are
in the order you found them. You'll need to organize them before they can be
analyzed.

What is the ID of the guard you chose multiplied by the minute you chose? (In
the above example, the answer would be 10 * 24 = 240.)
"""
from collections import Counter

import funcy as _

from common import get_guard_minutes_asleep, parse_event_list


def total_time_asleep(guard_id, guard_minutes_asleep):
    timings = guard_minutes_asleep[guard_id].values()
    return sum([len(mins) for mins in timings])


def get_guard_asleep_most(guard_minutes_asleep):
    all_times = {
        guard_id: total_time_asleep(guard_id, guard_minutes_asleep)
        for guard_id in guard_minutes_asleep.keys()
    }

    most_asleep_guard_id = max(all_times.keys(), key=(lambda k: all_times[k]))

    return (most_asleep_guard_id,
            list(guard_minutes_asleep[most_asleep_guard_id].values()))


if __name__ == "__main__":
    with open('input.txt') as guard_event_strings:
        sorted_guard_events = parse_event_list(guard_event_strings)

        guard_minutes_asleep = get_guard_minutes_asleep(sorted_guard_events)

        most_asleep_guard_id, minutes_asleep =\
            get_guard_asleep_most(guard_minutes_asleep)

        most_common_minute =\
            Counter(_.cat(daily_minutes
                          for daily_minutes in minutes_asleep))\
            .most_common(1)[0][0]

        print(most_asleep_guard_id * most_common_minute)
