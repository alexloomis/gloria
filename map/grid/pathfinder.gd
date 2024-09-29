extends Resource

class_name Pathfinder

# Is the tile clear?
# Does it have an ignorable obstruction? (E.g. I have trample.)
# Is the tile passible but not stoppable? (E.g. I have fly.)
# Is it blocked?
enum BLOCK_LEVEL {CLEAR = 0, IGNORE = 10, PASS_THROUGH = 20, BLOCKED = 30}

var _cells: Dictionary[Vector2i, CellInfo]
var _terrain_data: Dictionary[Terrain.TILE, int]

func update() -> void:
	_cells.clear()
	for x in Grid.cols:
		for y in Grid.rows:
			var cell: Vector2i = Vector2i(x,y)
			_cells[cell] = CellInfo.new()
			var tile: Terrain.TILE = Terrain.terrain[cell]
			_cells[cell].move_cost = _terrain_data.get(tile, Terrain.IMPASSIBLE)
			if Roster.roster.has(cell):
				_on_register(cell)
	Roster.registered.connect(_on_register)
	Roster.deregistered.connect(_on_deregister)

func _on_register(cell: Vector2i) -> void:
	block(cell)

func _on_deregister(cell: Vector2i) -> void:
	clear(cell)

func block(cell: Vector2i, block_level: BLOCK_LEVEL = BLOCK_LEVEL.BLOCKED) -> void:
	_cells[cell].block_level = block_level

func clear(cell: Vector2i) -> void:
	block(cell, BLOCK_LEVEL.CLEAR)

func is_blocked(cell: Vector2i, block_level: BLOCK_LEVEL = BLOCK_LEVEL.CLEAR) -> bool:
	return _cells[cell].block_level > block_level

func is_clear(cell: Vector2i, block_level: BLOCK_LEVEL = BLOCK_LEVEL.CLEAR) -> bool:
	return not is_blocked(cell, block_level)

func neighbors(cell: Vector2i, block_level: BLOCK_LEVEL = BLOCK_LEVEL.CLEAR) -> Array[Vector2i]:
	var ns: Array[Vector2i] = Grid.neighbors(cell)
	var _clear: Callable = func (v: Vector2i) -> bool:
		return is_clear(v, block_level)
	ns.filter(_clear)
	return ns

func cost(cell: Vector2i, block_level: BLOCK_LEVEL = BLOCK_LEVEL.CLEAR) -> int:
	if not _cells.has(cell):
		return Terrain.IMPASSIBLE
	if is_blocked(cell, block_level):
		return Terrain.IMPASSIBLE
	return _cells[cell].move_cost

@warning_ignore("shadowed_variable")
func total_cost(path: Array[Vector2i], block_level: BLOCK_LEVEL = BLOCK_LEVEL.CLEAR) -> int:
	var total: int = 0
	for cell: Vector2i in path.slice(1):
		total += cost(cell, block_level)
	return total

func distances(cell: Vector2i, block_level: BLOCK_LEVEL = BLOCK_LEVEL.CLEAR, max_cost: int = Terrain.IMPASSIBLE - 1) -> Dictionary[Vector2i, int]:
	var checked: Dictionary[Vector2i, int] = {}
	var reached: Dictionary[Vector2i, int] = {cell: 0}
	while not reached.is_empty():
		# Find cell with a minimal cost
		var best_cost: int = reached.values().min()
		var new_cell: Vector2i = reached.find_key(best_cost)
		# Compute costs to cell's reachable neighbors
		for n: Vector2i in neighbors(new_cell, block_level):
			if not n in checked:
				var c: int = best_cost + cost(n, block_level)
				if n in reached:
					if c < reached[n]: # know: reached[n] <= max_cost
						reached[n] = c
				elif c <= max_cost:
					reached[n] = c
		# Move cell to checked
		checked[new_cell] = best_cost
		reached.erase(new_cell)
	return checked

# block_level is the highest block level we consider to be clear
func path(from: Vector2i, to: Vector2i, block_level: BLOCK_LEVEL = BLOCK_LEVEL.CLEAR) -> Array[Vector2i]:
	var out: Array[Vector2i]
	var dists: Dictionary[Vector2i, int] = distances(to, block_level)
	if not dists.has(from):
		return out
	out.append(from)
	while out[-1] != to:
		var current: Vector2i = out[-1]
		var ns: Array[Vector2i] = neighbors(current, block_level)
		out.append(Util.min_among(ns, dists))
	return out
