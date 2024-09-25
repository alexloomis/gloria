extends Resource

class_name Navigator

var leader: Leader

var pf: Pathfinder:
	get:
		return leader.pf

func _targets(near: Vector2i) -> Array[Vector2i]:
	if not leader.formation:
		return []
	var ideal_targets: Array[Vector2i] = leader.formation.get_tiles(leader.followers.size() + 1).slice(1)
	for i in ideal_targets.size():
		ideal_targets[i] += near
	var targets: Array[Vector2i]
	var idx: int = 0
	for follower in leader.followers:
		var candidates: Array[Vector2i] = pf.costs(follower.cell, leader.speed).keys()
		var ideal_target: Vector2i = ideal_targets[idx]
		var path_to_ideal: PackedVector2Array = pf.find(follower.cell, ideal_target)
		var best_cost: int = pf.total_cost(path_to_ideal)
		var distances: Dictionary[Vector2i, int] = pf.costs(ideal_target, best_cost)
		var target: Vector2i = follower.cell
		for candidate in candidates:
			var cost: int = distances.get(candidate, TYPE_MAX)
			if cost < best_cost and not targets.has(candidate):
				target = candidate
				best_cost = cost
		targets.append(target)
		idx += 1
	return targets

func _sum(array: Array[int]) -> int:
	var acc: int = 0
	for x in array:
		acc += x
	return acc

func _nonnegative_at(array: Array[int]) -> int:
	for i in array.size():
		if array[i] >= 0:
			return i
	return -1

func _max_at(matrix: Array[Array]) -> Vector2i:
	var out: Vector2i
	var max_val: int = -1
	for x in matrix.size():
		for y in matrix[x].size():
			var val: int = matrix[x][y]
			if val > max_val:
				max_val = val
				out = Vector2i(x,y)
	return out

# Must have same number of units and targets!
func _assign_targets(targets: Array[Vector2i]) -> Dictionary[Follower, Vector2i]:
	var n: int = targets.size()
	assert(n == leader.followers.size())
	# Instead of manually checking how many possibilities remain, we can keep count
	#var num_avail_units: Array[int]
	var num_avail_targets: Array[int]
	num_avail_targets.resize(n)
	num_avail_targets.fill(n)
	# distance[follower][target]
	var distances: Array[Array]
	distances.resize(n)
	distances.fill(num_avail_targets)
	for f in n:
		for t in n:
			var path: PackedVector2Array = pf.find(leader.followers[f].cell, targets[t])
			distances[f][t] = pf.total_cost(path)
	var assignments: Dictionary[Follower, Vector2i]
	while true:
		if _sum(num_avail_targets) == 0:
			break
		# Which unit, if any, has exactly one available target?
		var unit_idx: int = num_avail_targets.find(1)
		if unit_idx > -1:
			var tgt_idx: int = _nonnegative_at(distances[unit_idx])
			assignments[leader.followers[unit_idx]] = targets[tgt_idx]
			distances[unit_idx][tgt_idx] = -1
			num_avail_targets[unit_idx] = 0
		else:
			# Remove the longest path from consideration
			var idx: Vector2i = _max_at(distances)
			distances[idx.x][idx.y] = -1
			num_avail_targets[idx.x] -= 1
	return assignments

func _await_movement() -> void:
	if leader.moving:
		await leader.finished_moving
	for follower in leader.followers:
		if follower.moving:
			await follower.finished_moving

func move_formation(to: Vector2i) -> void:
	if not leader.available:
		return
	leader.available = false
	var targets: Dictionary[Follower, Vector2i] = _assign_targets(_targets(to))
	print(targets)
	# Step by step, move self and followers
	for _step: int in range(0, leader.speed):
		# Unit -> Array[Vector2i]
		var preferences: Dictionary[Unit, Array]
		preferences[leader as Unit] = pf.closest_neighbors(leader.cell, to)
		for follower in leader.followers:
			preferences[follower as Unit] = pf.closest_neighbors(follower.cell, targets[follower])
		# Repeatedly try to move units. If no unit could move, go to second preference, etc.
		# Current method is if no one gets first choice, then everyone tries their second choice, etc.
		var unmoved: Array = preferences.keys()
		var choice: int = 0
		while not unmoved.is_empty():
			var successful_move: bool = false
			for unit: Unit in unmoved:
				var want: Vector2i = preferences[unit][choice]
				if want == unit.cell or pf.navigable(want):
					unit.move_adjacent(want)
					# It's fine that we erase from the array we're looping over, since we break instead of continuing the loop
					unmoved.erase(unit)
					successful_move = true
					# Maybe we've freed up space for an earlier unit.
					break
			if successful_move:
				choice = 0
			else:
				choice +=1
		await _await_movement()
		await leader.get_tree().create_timer(0.05).timeout
	leader.available = true

func move(to: Vector2i) -> void:
	if not leader.available:
		return
	leader.available = false
	var _targets_: Dictionary[Follower, Vector2i] = _assign_targets(_targets(to))
	# Step by step, move self and followers
	for _step: int in range(0, leader.speed):
		await _await_movement()
		await leader.get_tree().create_timer(0.05).timeout
	leader.available = true
