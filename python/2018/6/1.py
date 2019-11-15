"""
Using only the Manhattan distance, determine the area around each coordinate
by counting the number of integer X,Y locations that are closest to that
coordinate (and aren't tied in distance to any other coordinate).

Your goal is to find the size of the largest area that isn't infinite. For
example, consider the following list of coordinates:

1, 1
1, 6
8, 3
3, 4
5, 5
8, 9

If we name these coordinates A through F, we can draw them on a grid, putting
0,0 at the top left:

..........
.A........
..........
........C.
...D......
.....E....
.B........
..........
..........
........F.

This view is partial - the actual grid extends infinitely in all directions.
Using the Manhattan distance, each location's closest coordinate can be
determined, shown here in lowercase:

aaaaa.cccc
aAaaa.cccc
aaaddecccc
aadddeccCc
..dDdeeccc
bb.deEeecc
bBb.eeee..
bbb.eeefff
bbb.eeffff
bbb.ffffFf

Locations shown as . are equally far from two or more coordinates, and so they
don't count as being closest to any.

In this example, the areas of coordinates A, B, C, and F are infinite - while
not shown here, their areas extend forever outside the visible grid. However,
the areas of coordinates D and E are finite: D is closest to 9 locations, and E
is closest to 17 (both including the coordinate's location itself). Therefore,
in this example, the size of the largest area is 17.

What is the size of the largest area that isn't infinite?
"""
import math
from collections import defaultdict

from common import Point, iter_points


def find_closest_point(point, other_points):
    closest = -1
    closest_distance = math.inf

    for i, other_point in enumerate(other_points):
        if point == other_point:
            closest = i
            break

        distance = point.manhattan_disance(other_point)

        if distance == closest_distance:
            closest = -1
        elif distance < closest_distance:
            closest = i
            closest_distance = distance

    return closest


def find_belonging_areas(special_points):
    area_cnt = defaultdict(int)

    min_column = min(special_points, key=lambda p: p.from_left).from_left
    max_column = max(special_points, key=lambda p: p.from_left).from_left
    min_row = min(special_points, key=lambda p: p.from_top).from_top
    max_row = max(special_points, key=lambda p: p.from_top).from_top

    for point in iter_points(max_column, max_row):
        closest = find_closest_point(point, special_points)

        point_on_edge = (
            (point.from_left == min_column or point.from_left == max_column)
            or (point.from_top == min_row or point.from_top == max_row))

        # If it's not -1, it means it belongs to somebody
        if closest != -1:
            if point_on_edge:
                # This way, adding 1 to it will not matter. And it can be
                # disregarded when finding a maximum.
                area_cnt[closest] = -math.inf
                continue

            area_cnt[closest] += 1

    return area_cnt


if __name__ == "__main__":
    with open('input.txt') as points:
        points = list(map(Point.from_string, points.readlines()))
        areas = find_belonging_areas(points)

        print(max(areas.values()))
