import functools
import heapq
import operator as op
from collections import namedtuple
from enum import Enum


@functools.total_ordering
class Location(namedtuple("Location", ["x", "y"])):
    def __lt__(self, other):
        other = self._operand_converter(other)

        if self.y < other.y:
            return True
        elif other.y < self.y:
            return False

        if self.x < other.x:
            return True
        elif other.x < self.x:
            return False

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

    def __str__(self):
        return f"({self.x}, {self.y})"

    __repr__ = __str__


class EnvironmentTypes(Enum):
    WALL = '#'
    OPEN = '.'

    def __str__(self):
        return self.value


class Environment:
    def __init__(self, location, type_=EnvironmentTypes.OPEN):
        self.location = location

        if isinstance(type_, EnvironmentTypes):
            self.type_ = type_
        else:
            raise ValueError(f"Environment must have a valid type")

    def __str__(self):
        return f"{self.type_} @ {self.location}"

    __repr__ = __str__


class UnitTypes(Enum):
    ELF = 'E'
    GOBLIN = 'G'

    def __str__(self):
        return self.value


class Unit:
    def __init__(self, location, type_, hp=200, attack_power=3):
        self.location = location
        self._hp, self.attack_power = hp, attack_power

        if isinstance(type_, UnitTypes):
            self.type_ = type_
        else:
            raise ValueError(f"Unit must have a valid type")

    # Provide a way to break ties for the find_move function below. As the
    # description says, first the shortest paths matter, then the reading order
    # for the destination, then the reading order for the beginning.
    # Defined here to both make it clear it's not needed for the outside
    # program, and to avoid to overhead of making it every time find_move is
    # called.
    class CellNode(
            namedtuple("CellNode", ["distance", "cell", "starting_cell"])):
        def __lt__(self, other):
            if self == other:
                return False

            if self.distance == other.distance:
                if self.cell.location == other.cell.location:
                    return (self.starting_cell.location <
                            other.starting_cell.location)
                return self.cell.location < other.cell.location
            return self.distance < other.distance

        def __eq__(self, other):
            return (
                self.distance == other.distance
                and self.cell.location == other.cell.location and
                self.starting_cell.location == other.starting_cell.location)

    def find_move(self, surrounding_cells_func):
        frontier = []
        explored = {self}

        for cell in surrounding_cells_func(self):
            if not cell or cell.type_ != EnvironmentTypes.OPEN:
                if self.is_enemy(cell):
                    return None

                continue

            heapq.heappush(frontier, self.CellNode(1, cell, cell))

        while frontier:
            next_ = heapq.heappop(frontier)

            for cell in surrounding_cells_func(next_.cell):
                if not cell or cell in explored:
                    continue

                if cell.type_ != EnvironmentTypes.OPEN:
                    if self.is_enemy(cell):
                        return next_.starting_cell.location

                    continue

                heapq.heappush(
                    frontier,
                    self.CellNode(next_.distance + 1, cell,
                                  next_.starting_cell))
                explored.add(cell)

        return None

    def find_attack(self, surrounding_cells_func):
        surrounding_enemies =\
            [cell
             for cell in surrounding_cells_func(self)
             if self.is_enemy(cell)]

        if not surrounding_enemies:
            return None

        return min(
            surrounding_enemies, key=lambda cell: (cell.hp, cell.location))

    @property
    def hp(self):
        return self._hp

    @hp.setter
    def hp(self, value):
        if value > self._hp:
            raise ValueError(f"You cannot increase a unit's HP.")
        else:
            self._hp = value

    def is_enemy(self, other):
        return isinstance(other, Unit) and self.type_ != other.type_

    def __str__(self):
        return f"{self.type_}({self.hp}) @ {self.location}"

    __repr__ = __str__


class CombatGrid:
    def __init__(self, grid, units):
        self._grid, self.units = grid, units

    @classmethod
    def from_combat_grid_strings(cls, combat_grid_strings):
        grid = [[None for _ in range(len(row))] for row in combat_grid_strings]
        units = []

        for i, line in enumerate(combat_grid_strings):
            for j, char in enumerate(line):
                location = Location(j, i)

                if char in EnvironmentTypes._value2member_map_:
                    grid[i][j] = Environment(location, EnvironmentTypes(char))
                else:
                    units.append(Unit(location, UnitTypes(char)))
                    grid[i][j] = Environment(location)

        return cls(grid, units)

    def _attempt_attack(self, unit):
        possible_attack = unit.find_attack(self.get_adjacent_cells)

        if possible_attack:
            print(f"{unit} attacks {possible_attack}")

            possible_attack.hp -= unit.attack_power

            if possible_attack.hp <= 0:
                self.units.remove(possible_attack)

        return possible_attack

    def tick(self):
        for unit in sorted(self.units, key=op.attrgetter('location')):
            if unit not in self.units:
                continue

            if all(other_unit.type_ == unit.type_
                   for other_unit in self.units):
                return False

            if self._attempt_attack(unit):
                continue

            possible_move = unit.find_move(self.get_adjacent_cells)

            if possible_move:
                print(f"{unit} moves to {possible_move}")
                unit.location = possible_move
                self._attempt_attack(unit)

        return True

    def render(self):
        for line in self:
            units_in_row = []

            for item in line:
                if isinstance(item, Unit):
                    units_in_row.append(item)

                print(item.type_, end='')

            print("\t", end='')
            print(', '.join(map(str, units_in_row)))

    def get_adjacent_cells(self, cell):
        location = cell.location

        return (self.get((location.x, location.y - 1)),
                self.get((location.x, location.y + 1)),
                self.get((location.x - 1, location.y)),
                self.get((location.x + 1, location.y)))

    def __getitem__(self, key):
        if not isinstance(key, (tuple, list)):
            if key < 0:
                raise IndexError

            return [self[i, key] for i in range(len(self._grid[key]))]

        if any(i < 0 for i in key):
            raise IndexError

        try:
            return self.units[list(map(op.attrgetter('location'),
                                       self.units)).index(key)]
        except ValueError:
            return self._grid[key[1]][key[0]]

    def get(self, key, default=None):
        try:
            return self[key]
        except IndexError:
            return default
