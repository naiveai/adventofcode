from common import Point, iter_points


def find_equisum_region(special_points, max_sum):
    max_column = max(special_points, key=lambda p: p.from_left).from_left
    max_row = max(special_points, key=lambda p: p.from_top).from_top

    region_points = []

    for point in iter_points(max_column, max_row):
        sum_from_all_special_points = \
            sum(point.manhattan_disance(special_point)
                for special_point in special_points)

        if sum_from_all_special_points < max_sum:
            region_points.append(point)

    return region_points


if __name__ == "__main__":
    with open('input.txt') as points:
        points = list(map(Point.from_string, points.readlines()))
        equisum_region = find_equisum_region(points, max_sum=10000)
        print(len(equisum_region))
