extends Resource

class_name SeqPathfinder

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

func block(cell: Vector2i) -> void:
	_cells[cell].blocked = true

func reserve(cell: Vector2i, time: int) -> void:
	_cells[cell].block_times.append(time)

func clear(cell: Vector2i) -> void:
	_cells[cell].blocked = false
	_cells[cell].block_times.clear()

func is_blocked(cell: Vector2i, time: int = 0) -> bool:
	return _cells[cell].blocked or _cells[cell].block_times.has(time)

func is_clear(cell: Vector2i, time: int = 0) -> bool:
	return not is_blocked(cell, time)

func cost(cell: Vector2i) -> int:
	if not _cells.has(cell):
		return Terrain.IMPASSIBLE
	return _cells[cell].move_cost

# Assumes each vector after the zeroth is a new cell
func total_cost(path: Array[Vector2i]) -> int:
	var total: int = 0
	for cell: Vector2i in path.slice(1):
		total += cost(cell)
	return total

# Time coord is earliest departure time
func _neighbors(cell: Vector3i) -> Array[Vector3i]:
	var cell_: Vector2i = Util.project(cell)
	var time: int = cell.z
	var nbrs_: Array[Vector2i] = Grid.neighbors(cell_)
	var nbrs: Array[Vector3i]
	# If we wait a tick, our departure time is time + 1
	if is_clear(cell_, time + 1):
		nbrs.append(Util.wait(cell, 1))
	
	var to_be_removed: Array[int]
	for i in nbrs.size():
		for t in cost(nbrs_[i]):
			# If it's time 3 and the neighboring cell costs 2, it needs to be clear for time = 4, 5
			if is_blocked(nbrs_[i], time + t + 1):
				to_be_removed.append(i)
	to_be_removed.reverse()
	for i in to_be_removed:
		nbrs_.remove_at(i)
	
	for n_ in nbrs_:
		nbrs.append(Vector3i(n_.x, n_.y, time + cost(n_)))
	return nbrs

# Third coord is departure time.
# TODO: Stop when all targets are reached, if targets is non-empty
func _departure_times(origin: Vector3i, targets: Array[Vector2i] = [], max_time: int = Terrain.IMPASSIBLE - 1) -> Array[Vector3i]:
	# cell -> earliest *departure* time from cell
	# for clear neighbors of the origin, this is their cost
	var checked: Array[Vector3i] = []
	var reached: Array[Vector3i] = [origin]
	while not reached.is_empty():
		# Find cell with an earliest departure time
		var early_cell: Vector3i = Util.min_v3(reached)
		# Compute costs to cell's reachable neighbors
		for neighbor: Vector3i in _neighbors(early_cell):
			if (not neighbor in checked) and (not neighbor in reached):
				if neighbor.z <= max_time:
					reached.append(neighbor)
		# Move cell to checked
		checked.append(early_cell)
		reached.erase(early_cell)
		if not targets.is_empty():
			var all_reached: bool = true
			for target in targets:
				if not checked.has(target):
					all_reached = false
					break
			if all_reached:
				break
	return checked

func distances(origin: Vector2i, targets: Array[Vector2i] = [], max_dist: int = Terrain.IMPASSIBLE - 1) -> Dictionary[Vector2i, int]:
	var dep_times: Array[Vector3i] = _departure_times(Util.embed(origin), targets, max_dist)
	var out: Dictionary[Vector2i, int]
	for departure in dep_times:
		var cell: Vector2i = Util.project(departure)
		if not out.has(cell):
			out[cell] = departure.z
		elif departure.z < out[cell]:
			out[cell] = departure.z
	return out

# block_level is the highest block level we consider to be clear
func _find_path(from: Vector2i, to: Vector2i) -> Array[Vector3i]:
	var path: Array[Vector3i]
	var origin: Vector3i = Util.embed(from)
	var departure_times: Array[Vector3i] = _departure_times(origin)
	var is_dest: Callable = func(v: Vector3i) -> bool:
		return Util.project(v) == to
	var targets: Array[Vector3i] = departure_times.filter(is_dest)
	assert(targets.size() < departure_times.size(), "Need to duplicate?")
	var dest: Vector3i = Util.min_v3(targets)
	path.append(dest)
	while path[-1] != Util.embed(from):
		var current: Vector3i = path[-1]
		# If we can stand still, we do
		if departure_times.has(Util.wait(current, -1)):
			path.append(Util.wait(current, -1))
			continue
		var current_: Vector2i = Util.project(current)
		var target_time: int = current.z - cost(current_)
		var right_time: Callable = func(v: Vector3i) -> bool:
			return v.z == target_time
		var potential_origins: Array[Vector3i] = departure_times.filter(right_time)
		var neighbors: Array[Vector2i] = Grid.neighbors(current_)
		for orig in potential_origins:
			if neighbors.has(Util.project(orig)):
				path.append(orig)
				break
	path.reverse()
	return path

func _reserve_path(path: Array[Vector3i], total_time: int) -> void:
	for v: Vector3i in path.slice(1):
		var cell: Vector2i = Util.project(v)
		var delta: int = cost(cell)
		for time in range(v.z - delta + 1, v.z + 1):
			reserve(cell, time)
	for time in range(path[-1].z + 1, total_time + 1):
		reserve(Util.project(path[-1]), time)

func find_path(from: Vector2i, to: Vector2i, total_time: int) -> Array[Vector3i]:
	var path: Array[Vector3i] = _find_path(from, to)
	_reserve_path(path, total_time)
	return path
