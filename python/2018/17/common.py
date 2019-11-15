import itertools
import operator as op
from collections import namedtuple
from copy import copy

import attr


class Location(namedtuple("Location", ["x", "y"])):
    def __eq__(self, other):
        other = self._operand_converter(other)

        return (self.x == other.x) and (self.y == other.y)

    def _operand_converter(self, other):
        if isinstance(other, Location):
            return other
        elif isinstance(other, (tuple, list)):
            return Location(other[0], other[1])
        else:
            raise ValueError(
                f"Value of {type(other)} cannot be compared with a Location")

    def __hash__(self):
        return hash((self.x, self.y))

    def __str__(self):
        return f"({self.x}, {self.y})"

    __repr__ = __str__


@attr.s
class GroundGrid:
    clay = attr.ib(
        factory=frozenset, validator=attr.validators.instance_of(frozenset))

    flowing_water = attr.ib(
        factory=set,
        validator=attr.validators.optional(attr.validators.instance_of(set)))

    still_water = attr.ib(
        factory=set,
        validator=attr.validators.optional(attr.validators.instance_of(set)))

    def __attrs_post_init__(self):
        self._current_tips = copy(self.flowing_water)

        x_list = list(map(op.attrgetter('x'), self.clay))
        y_list = list(map(op.attrgetter('y'), self.clay))

        self._min_x, self._max_x =\
            min(x_list), max(x_list)

        self._min_y, self._max_y =\
            min(y_list), max(y_list)

    def tick(self):
        for water_tip in copy(self._current_tips):
            self._current_tips.discard(water_tip)

            up, down, left, right = self.get_adjacent_cells(water_tip)

            if self._flowable(down):
                self._current_tips.add(down)
                self.flowing_water.add(down)

                continue

            furthest_left, furthest_right =\
                (self._furthest_flowable_cell(left, -1),
                 self._furthest_flowable_cell(right, 1))

            row = frozenset(
                itertools.chain(
                    (Location(x, water_tip.y)
                     for x in range(water_tip.x, furthest_left[1].x - 1, -1)),
                    (Location(x, water_tip.y)
                     for x in range(water_tip.x + 1, furthest_right[1].x +
                                    1))))

            if furthest_left[0] or furthest_right[0]:
                self.flowing_water |= row

                if furthest_left[0]:
                    self._current_tips.add(furthest_left[1])

                if furthest_right[0]:
                    self._current_tips.add(furthest_right[1])
            else:
                self.still_water |= row
                self.flowing_water -= row
                self._current_tips.add(up)

    def _furthest_flowable_cell(self, cell, direction):
        if not self._flowable(cell):
            return (False, Location(cell.x - direction, cell.y))

        while True:
            _, down, left, right = self.get_adjacent_cells(cell)

            if self._flowable(down):
                return (True, cell)

            next_cell = left if direction == -1 else right

            if not self._flowable(next_cell):
                return (False, cell)

            cell = next_cell

    def is_out_of_bounds(self, cell):
        return not (self._min_y <= cell.y <= self._max_y)

    def _flowable(self, cell):
        return not (cell in self.clay or cell in self.still_water)

    def get_adjacent_cells(self, cell):
        return (Location(cell.x, cell.y - 1), Location(cell.x, cell.y + 1),
                Location(cell.x - 1, cell.y), Location(cell.x + 1, cell.y))

    def render(self):
        all_x = {
            *map(op.attrgetter('x'), self.flowing_water | self.still_water),
            self._min_x, self._max_x
        }

        all_y = {
            *map(op.attrgetter('y'), self.flowing_water | self.still_water),
            self._min_y, self._max_y
        }

        for y in range(min(all_y), max(all_y) + 1):
            for x in range(min(all_x), max(all_x) + 1):
                loc = (x, y)

                if loc in self.clay:
                    print(u'▓', end='')
                elif loc in self._current_tips:
                    print('+', end='')
                elif loc in self.still_water:
                    print('−', end='')
                elif loc in self.flowing_water:
                    print('¦', end='')
                else:
                    print(' ', end='')

            print()


def locations_from_range(coordinate_range_str):
    constant_term, varying_term = coordinate_range_str.strip().split(", ")
    constant_num = int(constant_term[2:])
    varying_min, varying_max = tuple(map(int, varying_term[2:].split("..")))
    varying_range = range(varying_min, varying_max + 1)

    return frozenset(
        Location(**{
            constant_term[0]: constant_num,
            varying_term[0]: varying_num
        }) for varying_num in varying_range)


def flow(ground):
    prev_num_water = 0

    while True:
        ground.tick()

        num_water = len(
            frozenset(
                filter(lambda c: not ground.is_out_of_bounds(c),
                       ground.flowing_water | ground.still_water)))

        if num_water == 0:
            continue

        if num_water == prev_num_water:
            return (ground, num_water)

        prev_num_water = num_water
