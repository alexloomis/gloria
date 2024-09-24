extends Shape

class_name Region

var _base_tiles: Array[Vector2i]

func get_shape() -> Array[Vector2i]:
	var arr: Array[Vector2i] = []
	for tile in _base_tiles:
		arr.append(_apply_transformation(tile))
	return arr
