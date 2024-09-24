extends Sprite2D

class_name Tile

@export var terrain: int

func _ready() -> void:
	centered = false
	scale = Grid.cell_size as Vector2 / texture.get_size()
