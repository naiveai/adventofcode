import operator as op
import re


def parse_point_info_str(point_info_str):
    parts_regex = re.compile(r"position=< *(?P<pos_x>-*\d+)," +
                             r" *(?P<pos_y>-*\d+) *>" +
                             r" *velocity=< *(?P<vel_x>-*\d+),"
                             r" *(?P<vel_y>-*\d+) *>")

    result = parts_regex.match(point_info_str)

    if not result:
        raise ValueError(
            f"String {point_info_str} doesn't look like a point info string")

    groups = result.groupdict()

    return ((int(groups['pos_x']), int(groups['pos_y'])),
            (int(groups['vel_x']), int(groups['vel_y'])))


def tick(point_info_list):
    def add_vel_to_point(point_info):
        pos, vel = point_info
        return (tuple(map(op.add, pos, vel)), vel)

    return list(map(add_vel_to_point, point_info_list))


def get_positions(point_info_list):
    return list(map(op.itemgetter(0), point_info_list))


def get_bounds(point_positions):
    min_row = min(point_positions, key=op.itemgetter(0))[0]
    max_row = max(point_positions, key=op.itemgetter(0))[0]
    min_column = min(point_positions, key=op.itemgetter(1))[1]
    max_column = max(point_positions, key=op.itemgetter(1))[1]

    return (min_row, max_row, min_column, max_column)


def render(point_info_list):
    point_positions = get_positions(point_info_list)

    min_row, max_row, min_column, max_column =\
        get_bounds(point_positions)

    for column in range(min_column, max_column + 1):
        for row in range(min_row, max_row + 1):
            if (row, column) in point_positions:
                print(u'▓', end='')
            else:
                print(u'░', end='')
        print()


def find_coherent_message(point_info_list):
    def area(bounds):
        min_row, max_row, min_column, max_column =\
            bounds
        return (max_row - min_row) * (max_column - min_column)

    prev_bounds = get_bounds(get_positions(point_info_list))
    prev_info_list = point_info_list

    seconds = 0

    while True:
        seconds += 1
        info_list = tick(prev_info_list)
        bounds = get_bounds(get_positions(info_list))

        if area(bounds) > area(prev_bounds):
            return (seconds - 1, prev_info_list)

        prev_info_list = info_list
        prev_bounds = bounds
