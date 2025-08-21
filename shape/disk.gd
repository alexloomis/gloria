extends Shape

class_name Disk

var radius: int:
	set(val):
		radius = max(val, 0)
		_update()

func _update() -> void:
	base_tiles = []
	for r in range(0, radius + 1):
		for tile in _square(r):
			if tile.length() <= radius:
				base_tiles.append(tile)
	size = base_tiles.size()

func _grow_shape() -> void:
	var desired_size: int = size
	while base_tiles.size() < size:
		radius += 1
	size = desired_size

func _init() -> void:
	_update()
