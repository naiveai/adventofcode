DIRECTIONS = {
    'N': (0, 1),
    'E': (1, 0),
    'S': (0, -1),
    'W': (-1, 0),
}

def parse_rooms_regex(rooms_regex):
    grid = {(0, 0): 0}
    dist = x = y = 0
    stack = []

    for char in rooms_regex:
        if char == '(':
            stack.append((dist, x, y))
        elif char == ')':
            dist, x, y = stack.pop()
        elif char == '|':
            dist, x, y = stack[-1]
        else:
            dx, dy = DIRECTIONS[char]
            x += dx
            y += dy

            dist += 1
            if (x, y) not in grid or dist < grid[(x, y)]:
                grid[(x, y)] = dist

    return grid
