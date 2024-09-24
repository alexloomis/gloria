extends Resource

class_name Pathfinder

var astar: AStarGrid2D = AStarGrid2D.new()
var terrain_data: Dictionary[Terrain.TILE, int]

func cost(cell: Vector2i) -> int:
	var type: Terrain.TILE = Terrain.terrain[cell]
	return terrain_data.get(type, -1)

func passable(cell: Vector2i) -> bool:
	if not Grid.in_bounds(cell):
		return false
	return cost(cell) > -1

# Call after changing Grid, region, cols, or rows
func update() -> void:
	astar.clear()
	astar.region = Rect2i(0, 0, Grid.cols, Grid.rows)
	astar.cell_size = Grid.cell_size
	astar.default_compute_heuristic = AStarGrid2D.HEURISTIC_MANHATTAN
	astar.default_estimate_heuristic = AStarGrid2D.HEURISTIC_MANHATTAN
	astar.diagonal_mode = AStarGrid2D.DIAGONAL_MODE_NEVER
	astar.update()
	for cell: Vector2i in Terrain.terrain:
		if cost(cell) > -1:
			astar.set_point_weight_scale(cell, cost(cell))
		else:
			astar.set_point_solid(cell)

func block(cell: Vector2i) -> void:
	astar.set_point_solid(cell)

func clear(cell: Vector2i) -> void:
	if passable(cell):
		astar.set_point_solid(cell, false)

func find(from: Vector2i, to: Vector2i) -> PackedVector2Array:
	return astar.get_id_path(from, to)

func find_px(from: Vector2i, to: Vector2i) -> PackedVector2Array:
	return astar.get_point_path(from, to)

func navigable(cell: Vector2i) -> bool:
	if Grid.in_bounds(cell):
		return not astar.is_point_solid(cell)
	else:
		return false

func neighbors(cell: Vector2i) -> Array[Vector2i]:
	return Grid.neighbors(cell)

func total_cost(path: PackedVector2Array) -> int:
	var total: int = 0
	for cell in path.slice(1):
		total += cost(cell)
	return total

func costs(from: Vector2i, max_cost: int = 100_000) -> Dictionary[Vector2i, int]:
	var checked: Dictionary[Vector2i, int] = {}
	var reached: Dictionary[Vector2i, int] = {from: 0}
	while not reached.is_empty():
		# Find cell with a minimal cost
		var best_cost: int = reached.values().min()
		var cell: Vector2i = reached.find_key(best_cost)
		# Compute costs to cell's reachable neighbors
		for n: Vector2i in neighbors(cell):
			if passable(n) and (not n in checked):
				var c: int = best_cost + cost(n)
				if n in reached:
					if c < reached[n]: # know: reached[n] <= max_cost
						reached[n] = c
				elif c <= max_cost:
					reached[n] = c
		# Move cell to checked
		checked[cell] = best_cost
		reached.erase(cell)
	return checked

# Returns array sorted closest to furthest.
func closest_neighbors(from: Vector2i, to: Vector2i, allow_self: bool = true) -> Array[Vector2i]:
	var ns: Array[Vector2i]
	if allow_self:
		ns.append(from)
	ns.append_array(neighbors(from).filter(passable))
	var costs_: Dictionary[Vector2i, int] = costs(to)
	ns.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return costs_[a] < costs_[b])
	return ns
