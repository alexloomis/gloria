extends Resource

class_name Navigator

var leader: VirtualUnit
var followers: Array[VirtualUnit]
var formation: Formation
# Reflects the positions of the virtual units.
var pf: Pathfinder

# TODO: Take movement costs into account for how far a unit can move.
# TODO: Support flying units for real.

func reset(real_leader: Leader) -> void:
	leader = VirtualUnit.new()
	leader.cell = real_leader.cell
	followers.clear()
	for follower in real_leader.followers:
		var vf: VirtualUnit = VirtualUnit.new()
		vf.cell = follower.cell
		vf.deficit = 0
		followers.append(vf)
	formation = real_leader.formation
	pf = real_leader.pf.duplicate()

func _block_own(block_level: Pathfinder.BLOCK_LEVEL) -> void:
	pf.block(leader.cell, block_level)
	for unit in followers:
		pf.block(unit.cell, block_level)

func _block_at_targets(block_level: Pathfinder.BLOCK_LEVEL = pf.BLOCK_LEVEL.BLOCKED) -> void:
	if leader.cell == leader.target:
		pf.block(leader.cell, block_level)
	for unit in followers:
		if unit.cell == unit.target:
			pf.block(unit.cell, block_level)

func _single_target(near: Vector2i, omit: Array[Vector2i]) -> Vector2i:
	var viable: Callable = func(v: Vector2i) -> bool:
		return pf.is_clear(v, pf.BLOCK_LEVEL.IGNORE) and not omit.has(v)
	if viable.call(near):
		return near
	var distances: Dictionary[Vector2i, int] = pf.distances(near, pf.BLOCK_LEVEL.PASS_THROUGH)
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
	# We don't want to take our own positions into account regarding what is clear.
	_block_own(pf.BLOCK_LEVEL.IGNORE)
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
	# Our path should not depend on anything we can pass through, nor that might move out of the way.
	# A unit on its current target might need to shift over and give up the spot,
	# so we do not block units on targets yet
	_block_own(pf.BLOCK_LEVEL.IGNORE)
	# distance[target][follower]
	var distances: Array[Array]
	for t in targets.size():
		var new: Array[int]
		new.resize(followers.size())
		for f in followers.size():
			var path: Array[Vector2i] = pf.path(followers[f].cell, targets[t], pf.BLOCK_LEVEL.PASS_THROUGH)
			new.append(pf.total_cost(path) + followers[f].deficit)
		distances.append(new)
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

#func _await_movement() -> void:
	#if leader.moving:
		#await leader.finished_moving
	#for follower in leader.followers:
		#if follower.moving:
			#await follower.finished_moving
#
#func move_formation(to: Vector2i) -> void:
	#if not leader.available:
		#return
	#leader.available = false
	#var targets: Dictionary[Follower, Vector2i] = _assign_targets(_targets(to))
	#print(targets)
	## Step by step, move self and followers
	#for _step: int in range(0, leader.speed):
		#await _await_movement()
		#await leader.get_tree().create_timer(0.05).timeout
	#leader.available = true

# If a unit steps here, can other units get out of the way?
func _valid_step(step: Vector2i) -> bool:
	if pf.is_clear(step):
		return true
	var checked: Array[Vector2i]
	var reached: Array[Vector2i] = [step]
	while not reached.is_empty():
		var cell: Vector2i = reached[0]
		for n in pf.neighbors(cell, pf.BLOCK_LEVEL.IGNORE):
			if pf.is_clear(n):
				return true
			if not (checked.has(n) or reached.has(n)):
				reached.append(n)
		checked.append(cell)
		reached.erase(cell)
	return false

func _follower_at(cell: Vector2i) -> VirtualUnit:
	for unit in followers:
		if unit.cell == cell:
			return unit
	return null

func _displace_followers() -> Array[VirtualUnit]:
	var unmoved: Array[VirtualUnit] = followers
	var displaced: VirtualUnit = _follower_at(leader.cell)
	while displaced:
		var ns: Array[Vector2i] = pf.neighbors(displaced.cell, pf.BLOCK_LEVEL.IGNORE)
		ns.filter(_valid_step)
		var distances: Dictionary[Vector2i, int] = pf.distances(displaced.target, pf.BLOCK_LEVEL.PASS_THROUGH)
		var cell: Vector2i = Util.min_among(ns, distances)
		var new_displaced: VirtualUnit = _follower_at(cell)
		displaced.cell = cell
		displaced.deficit += pf._terrain_data[Terrain.terrain[cell]]
		pf.block(cell) # So no one can get displaced here.
		unmoved.erase(displaced)
		displaced = new_displaced
	return unmoved

func _move_remainder(remainder: Array[VirtualUnit]) -> void:
	# Unit -> Array[Vector2i]
	var preferences: Dictionary[VirtualUnit, Array]
	for follower in remainder:
		var ns: Array[Vector2i] = pf.neighbors(follower.cell, pf.BLOCK_LEVEL.IGNORE)
		ns.append(follower.cell)
		var distances: Dictionary[Vector2i, int] = pf.distances(follower.cell, pf.BLOCK_LEVEL.PASS_THROUGH)
		preferences[follower] = Util.sort_with(ns, distances)
	# Repeatedly try to move units. If no unit could move, go to second preference, etc.
	# Current method is if no one gets first choice, then everyone tries their second choice, etc.
	var unmoved: Array = preferences.keys()
	var choice: int = 0
	while not unmoved.is_empty():
		var successful_move: bool = false
		for unit: VirtualUnit in unmoved:
			var want: Vector2i = preferences[unit][choice]
			if want == unit.cell or pf.is_clear(want):
				if want != unit.cell:
					unit.deficit += pf._terrain_data[Terrain.terrain[want]]
				pf.clear(unit.cell)
				unit.cell = want
				pf.block(unit.cell)
				# It's fine that we erase from the array we're looping over, since we break instead of continuing the loop
				unmoved.erase(unit)
				successful_move = true
				# Maybe we've freed up space for an earlier unit.
				break
		if successful_move:
			choice = 0
		else:
			choice +=1

# Given that the leader is taking this path,
# where will the followers be in the next step?
func _advance(path: Array[Vector2i]) -> void:
	# Where does everyone want to go?
	_assign_targets(_targets(path[-1]))
	# In case we remove the side effect later
	_block_own(pf.BLOCK_LEVEL.IGNORE)
	# The leader takes his step.
	# If he steps onto a unit, it tries to step, etc.
	# If somewhere down the line a unit cannot move, the first follower tries a different step, etc.
	# The follower is permitted to exchange places with the leader.
	pf.clear(leader.cell)
	leader.cell = path[0]
	pf.block(leader.cell)
	var unmoved: Array[VirtualUnit] = _displace_followers()
	# At this point anyone forced to move by the leader has moved, so we can assume anyone on their target is staying put.
	_block_at_targets()
	_move_remainder(unmoved)
	# Next we unblock the tiles the units are on, if they still haven't reached their target.

func find_paths(to: Vector2i, steps: int) -> Array[Array]:
	# Find leader's path, ignoring own units.
	var path: Array[Vector2i] = pf.path(leader.cell, to, pf.BLOCK_LEVEL.IGNORE)
	if path.size() > steps:
		path.resize(steps)
	elif path.size() < steps:
		for _i in steps - path.size():
			path.append(path[-1])
	var all_paths: Array[Array]
	all_paths.append([leader.cell])
	for unit in followers:
		all_paths.append([unit.cell])
	for step in steps:
		_advance(path.slice(step))
		all_paths[0].append(leader.cell)
		for i in followers.size():
			all_paths[i+1].append(followers[i].cell)
	return all_paths
