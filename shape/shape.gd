extends Resource

class_name Shape

@export var spread: int = 0:
	set(val):
		spread = max(val, 0)
# Mirrored then rotated
@export var mirror: bool = false
@export var rotation: int = 0:
	set(val):
		rotation = val % 8

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
func square(radius: int) -> Array[Vector2i]:
	var tiles: Array[Vector2i] = []
	if radius <= 0:
		tiles.append(Vector2i(0,0))
	else:
			# Ring has 8*radius tiles, so 2*radius _shifts starting in each cardinal direction
			for n in range(2*radius):
				for cell: Vector2i in [Vector2i(1,0), Vector2i(0,-1), Vector2i(-1,0), Vector2i(0,1)]:
					cell *= radius
					if n % 2 == 0:
						@warning_ignore("integer_division")
						tiles.append(_shift(cell, -n/2))
					else:
						@warning_ignore("integer_division")
						tiles.append(_shift(cell, (n+1)/2))
	return tiles
