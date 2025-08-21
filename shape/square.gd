extends Shape

class_name Square

@export var radius: int:
	set(val):
		radius = max(val, 0)
		_update()

func _update() -> void:
	base_tiles = _square(radius)

func _init() -> void:
	_update()

func _grow_shape() -> void:
	var desired_size: int = size
	while base_tiles.size() < size:
		radius += 1
	size = desired_size
