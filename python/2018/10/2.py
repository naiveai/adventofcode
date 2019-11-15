"""
Good thing you didn't have to wait, because that would have taken a long time -
much longer than the 3 seconds in the example above.

Impressed by your sub-hour communication capabilities, the Elves are curious:
exactly how many seconds would they have needed to wait for that message to
appear?
"""
from common import parse_point_info_str, find_coherent_message

if __name__ == "__main__":
    with open('input.txt') as point_info_str_list:
        point_info_list = list(map(parse_point_info_str, point_info_str_list))
        time_taken, _ = find_coherent_message(point_info_list)
        print(time_taken)
