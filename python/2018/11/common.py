import operator as op
from collections import namedtuple

Point = namedtuple("Point", ["x", "y"])


def get_power_level(point, grid_serial_number):
    rack_id = point.x + 10
    power_level = rack_id * point.y
    power_level += grid_serial_number
    power_level *= rack_id

    power_level_str = str(power_level)

    if len(power_level_str) >= 3:
        power_level = int(power_level_str[-3])
    else:
        power_level = 0

    power_level -= 5

    return power_level


def iter_all_points(grid_size):
    for column in range(1, grid_size[1] + 1):
        for row in range(1, grid_size[0] + 1):
            yield Point(column, row)


def iter_areas(grid_size, area_size):
    def get_area(point):
        for column in range(point.x, point.x + area_size[1]):
            for row in range(point.y, point.y + area_size[0]):
                if (column <= grid_size[1]) and (row <= grid_size[0]):
                    yield Point(column, row)

    for point in iter_all_points(grid_size):
        area = list(get_area(point))
        if len(area) == (area_size[0] * area_size[1]):
            yield area


def get_power_levels(grid_size, grid_serial_number):
    return {
        point: get_power_level(point, grid_serial_number)
        for point in iter_all_points(grid_size)
    }


def find_largest_total_power(grid_size, area_size, grid_serial_number):
    area_sums = {
        area[0]: sum(
            get_power_level(point, grid_serial_number)
            for point in area)
        for area in iter_areas(grid_size, area_size)
    }
    return max(area_sums.items(), key=op.itemgetter(1))
