import itertools
import re
from collections import namedtuple

Square = namedtuple("Square", ["row", "column"])


class Claim(
        namedtuple("Claim",
                   ["id", "from_left", "from_top", "width", "height"])):
    """Represents an individual Elf's claim to the fabric."""

    @classmethod
    def from_string(cls, str_):
        """Returns a claim object corresponding to the given string."""
        parts_regex = re.compile(r"\#(?P<id>\d+)" + r" *@ *" +
                                 r"(?P<from_left>\d+)" + r" *, *" +
                                 r"(?P<from_top>\d+)" + r" *: *" +
                                 r"(?P<width>\d+)x(?P<height>\d+)")

        result = parts_regex.match(str_)

        if not result:
            raise ValueError(
                f"String {str_} does not look like a claim string.")

        groups = {k: int(v) for k, v in result.groupdict().items()}

        return cls(**groups)

    def squares_occupied(self):
        """
        Returns a generator of the coordinates of all squares
        occupied by this claim.
        """
        top_left = Square(row=self.from_top, column=self.from_left)
        top_right = Square(
            row=top_left.row, column=(top_left.column + (self.width - 1)))
        bottom_left = Square(
            row=(top_left.row + (self.height - 1)), column=top_left.column)

        for column in range(top_left.column, top_right.column + 1):
            for row in range(top_left.row, bottom_left.row + 1):
                yield Square(row=row, column=column)


def claims_from_claim_strings(claim_strings):
    for claim_string in claim_strings:
        yield Claim.from_string(claim_string.strip())


def all_overlap_squares(claim_squares):
    for c1_sqs, c2_sqs in itertools.combinations(claim_squares, r=2):
        overlaps = frozenset.intersection(c1_sqs, c2_sqs)
        if overlaps:
            yield from overlaps
