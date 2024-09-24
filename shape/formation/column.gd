extends Formation

@export_range(1,100) var depth: int

func _get_tile(n: int) -> Vector2i:
	var x: int = n % depth
	@warning_ignore("integer_division")
	var row_idx: int = n / depth
	@warning_ignore("integer_division")
	var y: int = (row_idx + 1) / 2
	if row_idx % 2 == 0:
		y *= -1
	return Vector2i(x,y)
