"""
After a very long time, the water spring will run dry. How much water will be
retained?

In the example above, water that won't eventually drain out is shown as ~, a
total of 29 tiles.

How many water tiles are left after the water spring stops producing water and
all remaining water not at rest has drained?
"""
import itertools

from common import GroundGrid, Location, locations_from_range, flow

if __name__ == "__main__":
    with open('input.txt') as clay_coordinate_ranges:
        clay = frozenset(
            itertools.chain.from_iterable(
                map(locations_from_range, clay_coordinate_ranges.readlines())))

        ground = GroundGrid(clay, flowing_water={Location(500, 0)})

        print(len(flow(ground)[0].still_water))
