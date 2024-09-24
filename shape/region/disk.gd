extends Region

class_name Disk

var radius: int:
	set(val):
		radius = max(val, 0)
		update()

func update() -> void:
	_base_tiles = []
	for r in range(0, radius+1):
		for tile in square(r):
			if tile.length() <= radius: # + Grid.fudge:
				_base_tiles.append(tile)
