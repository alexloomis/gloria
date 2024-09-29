extends Resource

class_name SeqNavigator

var leader: VirtualUnit
var followers: Array[VirtualUnit]
var formation: Formation
# Reflects the positions of the virtual units.
var pf: SeqPathfinder
var speed: int

func reset(real_leader: Leader) -> void:
	leader = VirtualUnit.new()
	leader.cell = real_leader.cell
	followers.clear()
	for follower in real_leader.followers:
		var vf: VirtualUnit = VirtualUnit.new()
		vf.cell = follower.cell
		followers.append(vf)
	formation = real_leader.formation
	speed = real_leader.speed
	pf = real_leader.pf.duplicate()
	_clear_company()

func _clear_company() -> void:
	for cell in Roster.roster:
		if cell == leader.cell:
			pf.clear(cell)
		else:
			for unit in followers:
				if cell == unit.cell:
					pf.clear(cell)
					break

func _single_target(near: Vector2i, omit: Array[Vector2i]) -> Vector2i:
	var viable: Callable = func(v: Vector2i) -> bool:
		return pf.is_clear(v) and not omit.has(v)
	if viable.call(near):
		return near
	var distances: Dictionary[Vector2i, int] = pf.distances(near)
	var possibilities: Array[Vector2i] = distances.keys()
	possibilities.filter(viable)
	return Util.min_among(possibilities, distances)

# If units had infinite move, this is where the followers would end up.
func _targets(near: Vector2i) -> Array[Vector2i]:
	if not formation:
		return []
	# What the formation would be if everything were clear, clamped to be in bounds.
	var unassigned: Array[Vector2i] = formation.get_tiles(followers.size() + 1).slice(1)
	for i in unassigned.size():
		unassigned[i] += near
		unassigned[i] = Grid.clamp(unassigned[i])
	var targets: Array[Vector2i]
	for target in unassigned:
		targets.append(_single_target(target, targets))
	return targets

# If two followers wanted the same target, they might be assigned suboptimally.
# This minimizes the l^infty distance, assigns a target, then repeats.
func _assign_targets(targets: Array[Vector2i]) -> void:
	# Instead of manually checking how many possibilities remain, we can keep count
	var num_avail_targets: Array[int]
	num_avail_targets.resize(followers.size())
	num_avail_targets.fill(targets.size())
	var num_avail_units: Array[int]
	num_avail_units.resize(targets.size())
	num_avail_units.fill(followers.size())
	# distance[target][follower]
	var distances: Array[Array]
	for t in targets.size():
		var dist_to_target: Dictionary[Vector2i, int] = pf.distances(targets[t])
		var follower_dists: Array[int]
		follower_dists.resize(followers.size())
		for f in followers.size():
			follower_dists.append(dist_to_target[followers[f].cell])
		distances.append(follower_dists)
	while true:
		if Util.sum(num_avail_targets) == 0 or Util.sum(num_avail_units) == 0:
			break
		# Which target, if any, has exactly one available unit?
		var tgt_idx: int = num_avail_targets.find(1)
		var unit_idx: int = num_avail_units.find(1)
		if tgt_idx > -1:
			unit_idx = Util.nonnegative_at(distances[tgt_idx])
		elif unit_idx > -1:
			tgt_idx = Util.nonnegative_at(distances[unit_idx])
		if tgt_idx > -1 and unit_idx > -1: # Only really need to check one or the other
			followers[unit_idx].target = targets[tgt_idx]
			for n in targets.size():
				distances[n][unit_idx] = -1
			for n in followers.size():
				distances[tgt_idx][n] = -1
			num_avail_units[tgt_idx] = 0
			num_avail_targets[unit_idx] = 0
		else:
			# Remove the longest path from consideration
			var idx: Vector2i = Util.max_at(distances)
			distances[idx.x][idx.y] = -1
			num_avail_targets[idx.x] -= 1

func find_paths(to: Vector2i) -> Array[Array]:
	_assign_targets(_targets(to))
	var paths: Array[Array]
	paths.append(pf.find_path(leader.cell, to, speed))
	for follower in followers:
		paths.append(pf.find_path(follower.cell, follower.target, speed))
	return paths
