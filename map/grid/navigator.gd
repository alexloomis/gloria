extends Resource

class_name Navigator

var units: Array[VirtualUnit]
var formation: Shape
var pf: Pathfinder
var speed: int

func reset(real_leader: Leader) -> void:
	units.clear()
	units.append(VirtualUnit.new())
	units[0].cell = real_leader.cell
	units[0].idx = 0
	var idx: int = 1
	for follower in real_leader.followers:
		var vf: VirtualUnit = VirtualUnit.new()
		vf.cell = follower.cell
		vf.idx = idx
		idx += 1
		units.append(vf)
	formation = real_leader.formation
	speed = real_leader.speed
	pf = real_leader.pf
	_pf_clear_company()

func _pf_clear_company() -> void:
	for unit in units:
		pf.clear(unit.cell)

func _target_near(near: Vector2i, omit: Array[Vector2i]) -> Vector2i:
	var viable: Callable = func(v: Vector2i) -> bool:
		return pf.is_clear(v) and not omit.has(v)
	if viable.call(near):
		return near
	var distances: Dictionary[Vector2i, int] = pf.distances(near)
	var possibilities: Array[Vector2i] = distances.keys()
	possibilities = possibilities.filter(viable)
	return Util.min_among(distances, possibilities)

# If units had infinite move, this is where the units would end up.
func _targets(near: Vector2i) -> Array[Vector2i]:
	if not formation:
		printerr("no formation")
		return []
	# What the formation would be if everything were clear, clamped to be in bounds.
	var unassigned: Array[Vector2i] = formation.tiles
	for i in unassigned.size():
		unassigned[i] += near
		unassigned[i] = Grid.clamp(unassigned[i])
	var targets: Array[Vector2i]
	for target in unassigned:
		targets.append(_target_near(target, targets))
	return targets

func _assign_targets(followers: Array[VirtualUnit], targets: Array[Vector2i]) -> void:
	var distances: Dictionary[VirtualUnit, Dictionary]
	for unit in followers:
		var dists: Dictionary[Vector2i, int] = pf.distances(unit.cell)
		var target_dists: Dictionary[Vector2i, int]
		for target in targets:
			target_dists[target] = dists[target]
		distances[unit] = target_dists
	while not distances.is_empty():
		var max_: int = -1
		var max_unit: VirtualUnit
		var max_target: Vector2i
		for unit in distances:
			for target: Vector2i in distances[unit]:
				if distances[unit][target] > max_:
					max_ = distances[unit][target]
					max_unit = unit
					max_target = target
		var chosen_unit: VirtualUnit = max_unit
		var chosen_target: Vector2i = max_target
		var min_: int = max_
		for target: Vector2i in distances[max_unit]:
			if distances[max_unit][target] < min_:
				min_ = distances[max_unit][target]
				chosen_unit = max_unit
				chosen_target = target
		for unit in distances:
			if distances[unit][max_target] < min_:
				min_ = distances[unit][max_target]
				chosen_unit = unit
				chosen_target = max_target
		distances.erase(chosen_unit)
		for unit: VirtualUnit in distances:
			distances[unit].erase(chosen_target)
		chosen_unit.target = chosen_target

# Good enough for prototyping. Rewrite nicely in Rust later
func find_nonempty_paths(to: Vector2i) -> Array[Array]:
	var targets: Array[Vector2i] = _targets(to)
	units[0].target = targets[0]
	_assign_targets(units.slice(1), targets.slice(1))
	
	# Sort units by furthest to closest to their respective targets
	var target_dists: Dictionary[Vector2i, Dictionary]
	for target in targets:
		target_dists[target] = pf.distances(target)
	var unit_dists: Dictionary[VirtualUnit, int]
	for unit in units:
		# TODO: make units who can't move recognize it instead of crashing
		unit_dists[unit] = target_dists[unit.target][unit.cell]
	var f: Callable = func(u: VirtualUnit, v: VirtualUnit) -> bool:
		return unit_dists[u] < unit_dists[v]
	units.sort_custom(f)
	units.reverse()
	
	for try in range(10):
		for unit in units:
			var path: Array[Vector2i] = pf.find_path(unit.cell, unit.target, speed)
			if not path.is_empty():
				unit.path = path
				pf.reserve_path(path)
			else:
				units.erase(unit)
				units.push_front(unit)
				for a_unit in units:
					pf.release_path(a_unit.path)
					a_unit.path.clear()
				break
		var g: Callable = func(u: VirtualUnit, v: VirtualUnit) -> bool:
			return u.idx < v.idx
		units.sort_custom(g)
		var paths: Array[Array]
		for u in units:
			paths.append(u.path)
		return paths
	printerr("No valid path found in 10 tries")
	assert(false)
	return []
