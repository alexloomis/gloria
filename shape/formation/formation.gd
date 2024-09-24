extends Shape

class_name Formation

func get_tile(n: int) -> Vector2i:
	return _apply_transformation(_get_tile(n))

func get_tiles(n: int) -> Array[Vector2i]:
	return _get_tiles(n).map(_apply_transformation)

func _get_tile(n: int) -> Vector2i:
	return _get_tiles(n+1)[n]
 
func _get_tiles(n: int) -> Array[Vector2i]:
	var out: Array[Vector2i]
	for i in range(0,n):
		out.append(_get_tile(i))
	return out
