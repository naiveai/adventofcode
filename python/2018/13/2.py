"""
There isn't much you can do to prevent crashes in this ridiculous system.
However, by predicting the crashes, the Elves know where to be in advance and
instantly remove the two crashing carts the moment any crash occurs.

They can proceed like this for a while, but eventually, they're going to run
out of carts. It could be useful to figure out where the last cart that hasn't
crashed will end up.

For example:

/>-<\
|   |
| /<+-\
| | | v
\>+</ |
  |   ^
  \<->/

/---\
|   |
| v-+-\
| | | |
\-+-/ |
  |   |
  ^---^

/---\
|   |
| /-+-\
| v | |
\-+-/ |
  ^   ^
  \---/

/---\
|   |
| /-+-\
| | | |
\-+-/ ^
  |   |
  \---/

After four very expensive crashes, a tick ends with only one cart remaining;
its final location is 6,4.

What is the location of the last cart at the end of the first tick where it is
the only cart left?
"""
from common import TrackGrid

if __name__ == "__main__":
    with open('input.txt') as track_strs:
        track_list = track_strs.read().splitlines()
        track_grid = TrackGrid.from_track_list(track_list)

        while True:
            track_grid.tick(remove_duplicates=True)

            if len(track_grid.carts) == 1:
                print(track_grid.carts[0])
                break
