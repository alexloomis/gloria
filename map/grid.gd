extends Resource

class_name Grid

const cell_size := Vector2i(256, 256)
static var rows: int = 15
static var cols: int = 15
# A unit with movement m can move 100*m + fudge distance each turn.
# Also added to radii, etc. to make shapes more circular.
# const fudge: int = 43
# var terrain: Terrain

static func px_to_cell(v: Vector2i) -> Vector2i:
	return v / cell_size

static func cell_to_px(v: Vector2i) -> Vector2i:
	return v * cell_size

static func in_bounds(cell: Vector2i) -> bool:
		return cell.x >= 0 and cell.y >= 0 \
			and cell.x < cols and cell.y < rows

static func neighbors(cell: Vector2i) -> Array[Vector2i]:
	var out: Array[Vector2i]
	var dirs: Array[Vector2i] = [Vector2i(1,0), Vector2i(0,1), Vector2i(-1,0), Vector2i(0,-1)]
	for dir in dirs:
		dir += cell
		if in_bounds(dir):
			out.append(dir)
	return out

static func path_px(from: Vector2i, to: Vector2i) -> PackedVector2Array:
	if to == from:
		return []
	var from_px: Vector2i = cell_to_px(from)
	var to_px: Vector2i = cell_to_px(to)
	var path: PackedVector2Array
	var delta: Vector2i = to_px - from_px
	var delta_max: int = max(abs(delta.x), abs(delta.y))
	for t in range(0, delta_max + 1):
		var new_pt: Vector2 = (from_px as Vector2) + (t as float) / delta_max * (delta as Vector2)
		path.append(new_pt as Vector2i)
	return path
