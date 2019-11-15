from collections import Counter, namedtuple
from copy import copy
from enum import Enum

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


class CellTypes(Enum):
    OPEN = '.'
    TREE = '|'
    LUMBERYARD = '#'

    def __str__(self):
        return self.value


@attr.s
class Cell:
    type_ = attr.ib()
    location = attr.ib()

    def tick(self, surrounding_cells):
        surrounding_trees =\
            list(filter(lambda c: c and c.type_ == CellTypes.TREE,
                        surrounding_cells))

        if self.type_ == CellTypes.OPEN:
            if len(surrounding_trees) >= 3:
                self.type_ = CellTypes.TREE
        else:
            surrounding_yards =\
                list(filter(lambda c: c and c.type_ == CellTypes.LUMBERYARD,
                            surrounding_cells))

            if self.type_ == CellTypes.TREE:
                if len(surrounding_yards) >= 3:
                    self.type_ = CellTypes.LUMBERYARD
            else:
                if not (surrounding_yards and surrounding_trees):
                    self.type_ = CellTypes.OPEN


@attr.s
class LumberGrid:
    grid = attr.ib(factory=list)
    type_counts = attr.ib(factory=Counter, init=False)

    def __attrs_post_init__(self):
        for line in self:
            for cell in line:
                self.type_counts[cell.type_] += 1

    def tick(self):
        self.type_counts.clear()

        new_grid = [[None for _ in range(len(row))] for row in self.grid]

        for i, line in enumerate(self):
            for j, cell in enumerate(line):
                cell = copy(cell)
                cell.tick(self._get_surrounding_cells(cell))

                new_grid[i][j] = cell

                self.type_counts[cell.type_] += 1

        self.grid = new_grid

    def render(self):
        for line in self:
            for cell in line:
                print(str(cell.type_), end='')

            print()

    def _get_surrounding_cells(self, cell):
        loc = cell.location

        return [
            self.get((loc.x - 1, loc.y - 1)),
            self.get((loc.x, loc.y - 1)),
            self.get((loc.x + 1, loc.y - 1)),
            self.get((loc.x - 1, loc.y)),
            self.get((loc.x + 1, loc.y)),
            self.get((loc.x - 1, loc.y + 1)),
            self.get((loc.x, loc.y + 1)),
            self.get((loc.x + 1, loc.y + 1)),
        ]

    @classmethod
    def from_str_list(cls, str_list):
        grid = [[None for _ in range(len(row))] for row in str_list]

        for i, line in enumerate(str_list):
            for j, char in enumerate(line):
                grid[i][j] = Cell(
                    type_=CellTypes(char), location=Location(j, i))

        return cls(grid)

    def get(self, key, default=None):
        try:
            return self[key]
        except IndexError:
            return default

    def __getitem__(self, key):
        if not isinstance(key, (tuple, list)):
            if key < 0:
                raise IndexError

            return self.grid[key]

        if any(i < 0 for i in key):
            raise IndexError

        return self.grid[key[1]][key[0]]
