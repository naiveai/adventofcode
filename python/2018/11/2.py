"""
You discover a dial on the side of the device; it seems to let you select a
square of any size, not just 3x3. Sizes from 1x1 to 300x300 are supported.

Realizing this, you now must find the square of any size with the largest
total power. Identify this square by including its size as a third parameter
after the top-left coordinate: a 9x9 square with a top-left corner of 3,5 is
identified as 3,5,9.

For example:

For grid serial number 18, the largest total square (with a total power of 113)
is 16x16 and has a top-left corner of 90,269, so its identifier is 90,269,16.

For grid serial number 42, the largest total square (with a total power of 119)
is 12x12 and has a top-left corner of 232,251, so its identifier is 232,251,12.

What is the X,Y,size identifier of the square with the largest total power?
"""
from collections import defaultdict

serial = int(open("input.txt").read())
grid_sums, partial_sums = {}, defaultdict(int)

power_level = lambda x, y: ((((x + 10) * y + serial) * (x + 10)) // 10 ** 2 % 10) - 5
calculate_ps = lambda x, y: (power_level(x + 1, y + 1)
                             + partial_sums[x, y-1] + partial_sums[x-1, y] - partial_sums[x-1, y-1])

for j in range(300):
    for i in range(300):
        partial_sums[(i, j)] = calculate_ps(i, j)

for size in range(2, 300):
    for j in range(size-1, 300):
        for i in range(size-1, 300):
            gp = partial_sums[(i, j)] + partial_sums[(i-size, j-size)] \
                 - partial_sums[(i-size, j)] - partial_sums[(i, j-size)]
            grid_sums[gp] = (i-size+2, j-size+2, size)

print(grid_sums[max(grid_sums)])
