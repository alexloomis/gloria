extends Region

class_name Square

@export var radius: int:
	set(val):
		radius = max(val, 0)
		update()

func update() -> void:
	_base_tiles = square(radius)

func _init() -> void:
	update()
