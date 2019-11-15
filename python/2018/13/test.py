from common import TrackGrid

k = TrackGrid.from_track_list([">>--"])

k.tick(remove_duplicates=True)

k.render()
