"""
Each fuel cell has a coordinate ranging from 1 to 300 in both the X
(horizontal) and Y (vertical) direction. In X,Y notation, the top-left cell is
1,1, and the top-right cell is 300,1.

The interface lets you select any 3x3 square of fuel cells. To increase your
chances of getting to your destination, you decide to choose the 3x3 square
with the largest total power.

The power level in a given fuel cell can be found through the following
process:

- Find the fuel cell's rack ID, which is its X coordinate plus 10.
- Begin with a power level of the rack ID times the Y coordinate.
- Increase the power level by the value of the grid serial number (your puzzle
input).
- Set the power level to itself multiplied by the rack ID.
- Keep only the hundreds digit of the power level (so 12345 becomes 3; numbers
with no hundreds digit become 0).
- Subtract 5 from the power level.

Here are some example power levels:

- Fuel cell at  122,79, grid serial number 57: power level -5.
- Fuel cell at 217,196, grid serial number 39: power level  0.
- Fuel cell at 101,153, grid serial number 71: power level  4.

Your goal is to find the 3x3 square which has the largest total power. The
square must be entirely within the 300x300 grid. Identify this square using the
X,Y coordinate of its top-left fuel cell. For example:

For grid serial number 18, the largest total 3x3 square has a top-left corner
of 33,45 (with a total power of 29); these fuel cells appear in the middle of
this 5x5 region:

-2  -4   4   4   4
-4   4   4   4  -5
 4   3   3   4  -4
 1   1   2   4  -3
-1   0   2  -5  -2

For grid serial number 42, the largest 3x3 square's top-left is 21,61 (with a
total power of 30); they are in the middle of this region:

-3   4   2   2   2
-4   4   3   3   4
-5   3   3   4  -4
 4   3   3   4  -3
 3   3   3  -5  -1

What is the X,Y coordinate of the top-left fuel cell of the 3x3 square with the
largest total power?
"""
from common import find_largest_total_power

if __name__ == "__main__":
    with open('input.txt') as grid_serial_number:
        grid_serial_number = int(grid_serial_number.read().strip())
        print(find_largest_total_power((300, 300), (3, 3), grid_serial_number))
