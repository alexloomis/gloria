extends Node

const BIG_INT = 2**32

func min_among(dict: Dictionary[Vector2i, int], arr: Array[Vector2i] = dict.keys()) -> Vector2i:
	var vec: Vector2i = Vector2i(-1, -1)
	var best: int = BIG_INT
	for v in arr:
		if dict[v] < best:
			vec = v
			best = dict[v]
	return vec

func sum(array: Array[int]) -> int:
	var acc: int = 0
	for x in array:
		acc += x
	return acc

func nonnegative_at(array: Array[int]) -> int:
	for i in array.size():
		if array[i] >= 0:
			return i
	return -1

func max_at(matrix: Array[Array]) -> Vector2i:
	var out: Vector2i
	var max_val: int = -1
	for x in matrix.size():
		for y in matrix[x].size():
			var val: int = matrix[x][y]
			if val > max_val:
				max_val = val
				out = Vector2i(x,y)
	return out

func sort_with(arr: Array[Vector2i], dict: Dictionary[Vector2i, int]) -> Array[Vector2i]:
	var f: Callable = func(u: Vector2i, v: Vector2i) -> bool:
		return dict[u] < dict[v]
	arr.sort_custom(f)
	return arr

func project(v: Vector3i, dim: int = 2) -> Vector2i:
	match dim:
		0: return Vector2i(v.y, v.z)
		1: return Vector2i(v.x, v.z)
		_: return Vector2i(v.x, v.y)

func embed(v: Vector2i, dim: int = 2) -> Vector3i:
	match dim:
		0: return Vector3i(0, v.x, v.y)
		1: return Vector3i(v.x, 0, v.y)
		_: return Vector3i(v.x, v.y, 0) 

func wait(v: Vector3i, t: int) -> Vector3i:
	return v + Vector3i(0,0,t)

func min_v3(arr: Array[Vector3i]) -> Vector3i:
	var vec: Vector3i = arr[0]
	for v: Vector3i in arr.slice(1):
		if v.z < vec.z:
			vec = v
	return vec
