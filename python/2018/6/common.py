from collections import namedtuple


class Point(namedtuple("Point", ["from_left", "from_top"])):
    @classmethod
    def from_string(cls, str_):
        return cls(*tuple(map(int, str_.strip().split(", "))))

    def manhattan_disance(self, other):
        return (abs(self.from_left - other.from_left) +
                abs(self.from_top - other.from_top))


def iter_points(max_column, max_row):
    for from_top in range(max_row + 1):
        for from_left in range(max_column + 1):
            yield Point(from_left, from_top)
