import functools
import itertools
import operator as op
from collections import namedtuple
from enum import Enum


class CartDirections(Enum):
    UP = '^'
    DOWN = 'v'
    LEFT = '<'
    RIGHT = '>'

    def __str__(self):
        return self.value

    def equivalent_celltype(self):
        if self in (self.UP, self.DOWN):
            return CellTypes.VERTICAL
        else:
            return CellTypes.HORIZONTAL


class CellTypes(Enum):
    NONE = ' '
    VERTICAL = '|'
    HORIZONTAL = '-'
    FORWARD_CURVE = '/'
    BACKWARD_CURVE = '\\'
    INTERSECTION = '+'

    def __str__(self):
        return self.value


Cell = namedtuple("Cell", ["location", "type_"])


@functools.total_ordering
class Cart:
    def __init__(self, location, direction):
        self.location, self.direction = location, direction

        self._intersection_loop = [
            CartDirections.LEFT, None, CartDirections.RIGHT
        ]
        self._next_intersection_direction_index = 0

    def tick(self, surrounding_points):
        self.location, new_cell_type = {
            CartDirections.UP: surrounding_points[0],
            CartDirections.DOWN: surrounding_points[1],
            CartDirections.LEFT: surrounding_points[2],
            CartDirections.RIGHT: surrounding_points[3]
        }[self.direction]

        self.direction = self._get_direction_for_cell(new_cell_type)

    def _get_direction_for_cell(self, cell_type):
        if cell_type == CellTypes.NONE:
            raise ValueError("Cannot travel with no tracks")

        elif cell_type == CellTypes.VERTICAL:
            if self.direction in (CartDirections.UP, CartDirections.DOWN):
                return self.direction
            else:
                raise ValueError(
                    f"Cannot go {self.direction.name} on a vertical")

        elif cell_type == CellTypes.HORIZONTAL:
            if self.direction in (CartDirections.LEFT, CartDirections.RIGHT):
                return self.direction
            else:
                raise ValueError(
                    f"Cannot go {self.direction.name} on a horizontal")

        elif cell_type == CellTypes.FORWARD_CURVE:
            return {
                CartDirections.UP: CartDirections.RIGHT,
                CartDirections.DOWN: CartDirections.LEFT,
                CartDirections.LEFT: CartDirections.DOWN,
                CartDirections.RIGHT: CartDirections.UP
            }[self.direction]

        elif cell_type == CellTypes.BACKWARD_CURVE:
            return {
                CartDirections.UP: CartDirections.LEFT,
                CartDirections.DOWN: CartDirections.RIGHT,
                CartDirections.LEFT: CartDirections.UP,
                CartDirections.RIGHT: CartDirections.DOWN
            }[self.direction]

        elif cell_type == CellTypes.INTERSECTION:
            intersection_direction =\
                self._intersection_loop[
                    self._next_intersection_direction_index]

            self._next_intersection_direction_index =\
                (self._next_intersection_direction_index + 1)\
                % len(self._intersection_loop)

            if not intersection_direction:
                return self.direction
            elif intersection_direction == CartDirections.LEFT:
                return {
                    CartDirections.UP: CartDirections.LEFT,
                    CartDirections.DOWN: CartDirections.RIGHT,
                    CartDirections.LEFT: CartDirections.DOWN,
                    CartDirections.RIGHT: CartDirections.UP
                }[self.direction]
            elif intersection_direction == CartDirections.RIGHT:
                return {
                    CartDirections.UP: CartDirections.RIGHT,
                    CartDirections.DOWN: CartDirections.LEFT,
                    CartDirections.LEFT: CartDirections.UP,
                    CartDirections.RIGHT: CartDirections.DOWN
                }[self.direction]

    def __lt__(self, other):
        if self == other:
            return False

        other_location = self._comparision_operand_converter(other)

        other_further_right = self.location[0] < other_location[0]
        other_further_down = self.location[1] < other_location[1]

        return other_further_down or other_further_right

    def __eq__(self, other):
        return self.location == self._comparision_operand_converter(other)

    def __repr__(self):
        return f"Cart at {self.location} going {self.direction.name}"

    def _comparision_operand_converter(self, other):
        if type(other) in (tuple, list):
            other_location = other
        elif hasattr(other, 'location'):
            other_location = other.location
        else:
            other_location = None

        return other_location


class TrackGrid:
    def __init__(self, grid, carts):
        self.grid, self.carts = grid, carts

    @classmethod
    def from_track_list(cls, track_list):
        grid = [[None for _ in range(len(track_list[row]))]
                for row in range(len(track_list))]
        carts = []

        for i, line in enumerate(track_list):
            for j, char in enumerate(line):
                location = (j, i)

                if char in CartDirections._value2member_map_:
                    direction = CartDirections(char)

                    carts.append(Cart(location, direction))

                    grid[i][j] = Cell(location,
                                      direction.equivalent_celltype())
                else:
                    grid[i][j] = Cell(location, CellTypes(char))

        return cls(grid, carts)

    def tick(self, remove_duplicates=False):
        crashed_carts = []

        for cart in sorted(self.carts):
            if cart in crashed_carts:
                continue

            cart.tick(self._get_surrounding_points(cart.location))

            index = self.carts.index(cart)
            carts_without_current =\
                [c for i, c in enumerate(self.carts) if i != index]

            if cart in carts_without_current:
                crashed_carts.append(cart)
                crashed_carts.append(
                    carts_without_current[carts_without_current.index(cart)])

        if remove_duplicates:
            self.carts =\
                [c for c in self.carts if c not in crashed_carts]

        return crashed_carts

    def render(self, show_carts=True):
        for i, row in enumerate(self):
            for j, char in enumerate(row):
                if show_carts:
                    try:
                        cart_index =\
                            self.carts.index((j, i))

                        print(self.carts[cart_index].direction, end='')
                        continue
                    except ValueError:
                        pass

                print(char.type_, end='')
            print(end="\n")

    def _get_surrounding_points(self, location):
        return (self.get((location[0], location[1] - 1)),
                self.get((location[0], location[1] + 1)),
                self.get((location[0] - 1, location[1])),
                self.get((location[0] + 1, location[1])))

    def __getitem__(self, key):
        if type(key) not in (tuple, list):
            return self.grid[key]

        return self.grid[key[1]][key[0]]

    def get(self, key, default=None):
        try:
            return self[key]
        except IndexError:
            return default
