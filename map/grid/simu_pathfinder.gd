extends Resource

class_name Pathfinder

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

func clear(cell: Vector2i) -> void:
	_cells[cell].blocked = false

func is_blocked(cell: Vector2i) -> bool:
	return _cells[cell].blocked

func is_clear(cell: Vector2i) -> bool:
	return cost(cell) < Terrain.IMPASSIBLE and not is_blocked(cell)

func cost(cell: Vector2i) -> int:
	if not _cells.has(cell):
		return Terrain.IMPASSIBLE
	return _cells[cell].move_cost

# Assumes each vector after the zeroth is a new cell
func path_cost(path: Array[Vector2i]) -> int:
	var total: int = 0
	for cell: Vector2i in path.slice(1):
		total += cost(cell)
	return total

func distances(origin: Vector2i, max_dist: int = Terrain.IMPASSIBLE - 1) -> Dictionary[Vector2i, int]:
	var checked: Dictionary[Vector2i, int]
	var reached: Dictionary[Vector2i, int] = {origin: 0}
	while not reached.is_empty():
		# Find cell with an earliest departure time
		var early_cell: Vector2i = Util.min_among(reached)
		# Compute costs to cell's reachable neighbors
		for neighbor: Vector2i in Grid.neighbors(early_cell):
			var new_cost: int = reached[early_cell] + cost(neighbor)
			if new_cost < max_dist and not neighbor in checked:
				if neighbor in reached:
					reached[neighbor] = min(reached[neighbor], new_cost)
				else:
					reached[neighbor] = new_cost
		# Move cell to checked
		checked[early_cell] = reached[early_cell]
		reached.erase(early_cell)
	return checked

func _neighbor_builder(cell: Vector2i, states: Array[Dictionary]) -> Array[Dictionary]:
	var neighbors: Dictionary[Vector2i, int]
	neighbors[cell] = 0
	for neighbor: Vector2i in Grid.neighbors(cell).filter(is_clear):
		neighbors[neighbor] = cost(neighbor)
	var new_states: Array[Dictionary]
	for state: Dictionary[Vector2i, int] in states:
		for neighbor in neighbors:
			if not neighbor in state:
				var new_state: Dictionary[Vector2i, int] = state
				new_state[neighbor] = neighbors[neighbor]
				assert(state.size() < new_state.size(), "Ensure not appending to state.")
				new_states.append(new_state)
	return new_states

func _neighbors(state: Dictionary[Vector2i, int]) -> Array[Dictionary]:
	# The cells that MUST be stationary
	var stationary: Dictionary[Vector2i, int]
	for cell in state:
		if state[cell] > 0:
			stationary[cell] = state[cell] - 1
	var states: Array[Dictionary] = [stationary]
	for cell in state:
		if state[cell] == 0:
			states = _neighbor_builder(cell, states)
	return states

func _align_to_targets(cells: Array[Vector2i], targets: Array[Vector2i]) -> Dictionary[Vector2i, Vector2i]:
	var distances_: Dictionary[Vector2i, Dictionary]
	var assignments: Dictionary[Vector2i, Vector2i]
	for unit in cells:
		var dists: Dictionary[Vector2i, int] = distances(unit)
		var target_dists: Dictionary[Vector2i, int]
		for target in targets:
			target_dists[target] = dists[target]
		distances_[unit] = target_dists
	while not distances_.is_empty():
		var max_: int = -1
		var max_unit: Vector2i
		var max_target: Vector2i
		for unit in distances_:
			for target: Vector2i in distances_[unit]:
				if distances_[unit][target] > max_:
					max_ = distances_[unit][target]
					max_unit = unit
					max_target = target
		var chosen_unit: Vector2i = max_unit
		var chosen_target: Vector2i = max_target
		var min_: int = max_
		for target: Vector2i in distances_[max_unit]:
			if distances_[max_unit][target] < min_:
				min_ = distances_[max_unit][target]
				chosen_unit = max_unit
				chosen_target = target
		for unit in distances_:
			if distances_[unit][max_target] < min_:
				min_ = distances_[unit][max_target]
				chosen_unit = unit
				chosen_target = max_target
		distances_.erase(chosen_unit)
		for unit: Vector2i in distances_:
			distances_[unit].erase(chosen_target)
		assignments[chosen_unit] = chosen_target
	return assignments

func _heuristic(cells: Array[Vector2i], targets: Array[Vector2i]) -> int:
	var assignments: Dictionary[Vector2i, Vector2i] = _align_to_targets(cells, targets)
	var total: int = 0
	for unit in cells:
		var dists: Dictionary[Vector2i, int] = distances(unit)
		total += dists[assignments[unit]]
	return total

func _total_cost(state: Dictionary[Vector2i, int], new_state: Dictionary[Vector2i, int]) -> int:
	var total: int = 0
	var stationary: Array[Vector2i]
	for cell in state:
		if state[cell] > 0 or new_state[cell] == 0:
			stationary.append(cell)
	for cell in stationary:
		new_state.erase(cell)
	for cell in new_state:
		total += new_state[cell]
	return total

func _min_with_heuristic(states: Dictionary[Dictionary, int], targets: Array[Vector2i]) -> Dictionary[Vector2i, int]:
	var out: Dictionary[Vector2i, int]
	var best: int = Util.BIG_INT
	for v in states.keys():
		var estimate: int = states[v] + _heuristic(v.keys(), targets)
		if estimate < best:
			out = v
			best = estimate
	return out

func _find_paths(state: Dictionary[Vector2i, int], targets: Array[Vector2i]):
	var checked: Dictionary[Dictionary, int]
	var reached: Dictionary[Dictionary, int] = {state: 0}
	while not reached.is_empty():
		# Find cell with an earliest departure time
		var next: Dictionary[Vector2i, int] = _min_with_heuristic(reached, targets)
		# Compute costs to cell's reachable neighbors
		for neighbor: Dictionary[Vector2i, int] in _neighbors(next):
			var new_cost: int = reached[next] + _total_cost(next, neighbor)
			var est_cost: int = new_cost + _heuristic(neighbor.keys(), targets)
			if est_cost < best and not neighbor in checked:
				if neighbor in reached:
					reached[neighbor] = min(reached[neighbor], new_cost)
				else:
					reached[neighbor] = new_cost
		# Move cell to checked
		checked[early_cell] = reached[early_cell]
		reached.erase(early_cell)
	return checked
