import re
from collections import Counter, defaultdict, namedtuple
from datetime import datetime
from functools import total_ordering


class EventType(namedtuple("EventType", ["type_num", "id"])):
    @classmethod
    def BEGINS_SHIFT(cls, id_):
        return cls(0, id_)

    @classmethod
    def FALLS_ASLEEP(cls):
        return cls(1, None)

    @classmethod
    def WAKES_UP(cls):
        return cls(2, None)

    def __eq__(self, other):
        return self.type_num == other.type_num

    @classmethod
    def from_string(cls, str_):
        str_ = str_.strip()
        begins_shift_match = re.match(r"Guard #(\d+) begins shift", str_)

        if begins_shift_match:
            return cls.BEGINS_SHIFT(int(begins_shift_match.groups()[0]))
        if str_ == "falls asleep":
            return cls.FALLS_ASLEEP()
        if str_ == "wakes up":
            return cls.WAKES_UP()

        raise ValueError(
            "String {str_} does not look like an event type string")


@total_ordering
class Event(namedtuple("Event", ["datetime", "type"])):
    @classmethod
    def from_string(cls, str_):
        str_ = str_.strip()

        parts_regex = re.compile(r"\[(?P<datetime_str>.*)\] *(?P<type_str>.*)")

        result = parts_regex.match(str_)

        if not result:
            raise ValueError(
                f"String {str_} does not look like an event string.")

        group_strings = result.groupdict()
        groups = {
            'datetime':
            datetime.strptime(group_strings['datetime_str'], "%Y-%m-%d %H:%M"),
            'type':
            EventType.from_string(group_strings['type_str'])
        }

        return cls(**groups)

    def __eq__(self, other):
        return ((self.datetime == other.datetime)
                and (self.type == other.type))

    def __lt__(self, other):
        return self.datetime < other.datetime


def parse_event_list(event_strings):
    events = []
    for event_string in event_strings:
        events.append(Event.from_string(event_string))
    events.sort()
    return events


def get_guard_minutes_asleep(events):
    current_date = events[0].datetime.date()
    current_guard = None
    last_asleep = None
    guard_timings = defaultdict(lambda: defaultdict(set))

    for event in events:
        if event.type == EventType.BEGINS_SHIFT(None):
            current_guard = event.type.id
            continue

        event_date = event.datetime.date()
        if event_date != current_date:
            current_date = event_date

        if event.type == EventType.FALLS_ASLEEP():
            last_asleep = event.datetime.time().minute
        elif event.type == EventType.WAKES_UP():
            guard_timings[current_guard][current_date]\
                .update(range(last_asleep, event.datetime.time().minute))

    return guard_timings
