extends Resource

class_name Shape

@export var size: int = 0:
	set(val):
		size = max(val, 0)
		if size > base_tiles.size():
			_grow_shape()
# Mirrored then rotated
@export var mirror: bool = false
@export var rotation: int = 0:
	set(val):
		rotation = val % 8
@export var spread: int = 0:
	set(val):
		spread = max(val, 0)

var base_tiles: Array[Vector2i]:
	set(base):
		base_tiles = base
var tiles: Array[Vector2i]:
	get:
		var out: Array[Vector2i] = []
		out.resize(size)
		for i in size:
			out[i] = _apply_transformation(base_tiles[i])
		return out

# Override this to create different shapes
func _grow_shape() -> void:
	pass

func _shift(tile: Vector2i, n: int = 1) -> Vector2i:
	var r: int = max(tile.abs().x, tile.abs().y)
	# Ring has 8*r tiles
	n %= 8*r
	if n == 0:
		return tile
	if tile.x == r and not tile.y == -r:
		tile += Vector2i(0,-1)
	elif tile.y == r and not tile.x == r:
		tile += Vector2i(1,0)
	elif tile.x == -r and not tile.y == r:
		tile += Vector2i(0,1)
	elif tile.y == -r and not tile.x == -r:
		tile += Vector2i(-1,0)
	return(_shift(tile, n-1))

func _rotate(tile: Vector2i) -> Vector2i:
	var radius: int = max(tile.abs().x, tile.abs().y)
	var steps: int = rotation * radius
	for s in range(steps):
		tile = _shift(tile)
	return tile

func _apply_transformation(tile: Vector2i) -> Vector2i:
	if mirror:
		tile.y = -tile.y
	tile = _rotate(tile)
	if spread > 0:
		tile *= spread
	return tile

# Common functions to help define shapes

# Cc-wise, balanced sides
func _square(radius: int) -> Array[Vector2i]:
	var sq_tiles: Array[Vector2i] = []
	if radius <= 0:
		sq_tiles.append(Vector2i(0,0))
	else:
			# Ring has 8*radius tiles, so 2*radius _shifts starting in each cardinal direction
			for n in range(2*radius):
				for cell: Vector2i in [Vector2i(1,0), Vector2i(0,-1), Vector2i(-1,0), Vector2i(0,1)]:
					cell *= radius
					if n % 2 == 0:
						@warning_ignore("integer_division")
						sq_tiles.append(_shift(cell, -n/2))
					else:
						@warning_ignore("integer_division")
						sq_tiles.append(_shift(cell, (n+1)/2))
	return sq_tiles
