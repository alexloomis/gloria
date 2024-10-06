extends Resource

class_name Navigator

var units: Array[VirtualUnit]
var formation: Formation
# Reflects the positions of the virtual units.
var pf: Pathfinder
var speed: int

func reset(real_leader: Leader) -> void:
	units.clear()
	units.append(VirtualUnit.new())
	units[0].cell = real_leader.cell
	for follower in real_leader.followers:
		var vf: VirtualUnit = VirtualUnit.new()
		vf.cell = follower.cell
		units.append(vf)
	formation = real_leader.formation
	speed = real_leader.speed
	pf = real_leader.pf
	_clear_company()

func _clear_company() -> void:
	for unit in units:
		pf.clear(unit.cell)

func _single_target(near: Vector2i, omit: Array[Vector2i]) -> Vector2i:
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
		return []
	# What the formation would be if everything were clear, clamped to be in bounds.
	var unassigned: Array[Vector2i] = formation.get_tiles(units.size())
	for i in unassigned.size():
		unassigned[i] += near
		unassigned[i] = Grid.clamp(unassigned[i])
	var targets: Array[Vector2i]
	for target in unassigned:
		targets.append(_single_target(target, targets))
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

func find_paths(to: Vector2i) -> Array[Array]:
	var targets: Array[Vector2i] = _targets(to)
	units[0].target = targets[0]
	_assign_targets(units.slice(1), targets.slice(1))
	var paths: Array[Array]
	for unit in units:
		paths.append(pf.find_path(unit.cell, unit.target, speed))
	return paths
