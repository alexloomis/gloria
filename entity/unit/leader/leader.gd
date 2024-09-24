extends Unit

class_name Leader

@export_range(0,10) var speed: int
@export var formation: Formation
@export var follower_data: Array[PackedScene]
@export var terrain_data: Dictionary[Terrain.TILE, int]

var animation_speed: int:
	get:
		return (Settings.animation_speed as float) * sqrt(speed as float) as int
var available: bool
var followers: Array[Follower]
var pf: Pathfinder

func _ready() -> void:
	super()
	_pf_init()
	Roster.registered.connect(_on_registered)
	Roster.deregistered.connect(_on_deregistered)
	_place_followers()
	available = true
	leader = self

func _pf_init() -> void:
	pf = Pathfinder.new()
	pf.terrain_data = terrain_data
	pf.update()
	# Block occupied tiles.
	for loc: Vector2i in Roster.roster:
		pf.block(loc)

func _instantiate_followers() -> void:
	followers.clear()
	for scene in follower_data:
		var new_follower: Follower = scene.instantiate() as Follower
		followers.append(new_follower)
		new_follower.leader = self

func _place_followers() -> void:
	if not formation:
		return
	_instantiate_followers()
	var idx: int = 0
	for follower in followers:
		var placed: bool = false
		while not placed:
			var candidate: Vector2i = cell + formation.get_tile(idx)
			if pf.navigable(candidate):
				follower.cell = candidate
				placed = true
			idx += 1
	for follower in followers:
		add_sibling.call_deferred(follower)

func _on_registered(entity: Unit) -> void:
	pf.block(entity.cell)

func _on_deregistered(loc: Vector2i) -> void:
	pf.clear(loc)

func _movement_targets(near: Vector2i) -> Array[Vector2i]:
	if not formation:
		return []
	var targets: Array[Vector2i]
	var shape_idx: int = 1
	for _follower in followers:
		var target_found: bool = false
		while not target_found:
			var candidate: Vector2i = near + formation.get_tile(shape_idx)
			shape_idx += 1
			if pf.passable(candidate):
				targets.append(candidate)
				target_found = true
				break
	return targets

func _await_movement() -> void:
	if moving:
		await finished_moving
	for follower in followers:
		if follower.moving:
			await follower.finished_moving

func move_formation(to: Vector2i) -> void:
	if not available:
		return
	available = false
	var targets: Array[Vector2i] = _movement_targets(to)
	# Step by step, move self and followers
	for _step: int in range(0, speed):
		# Unit -> Array[Vector2i]
		var preferences: Dictionary[Unit, Array]
		preferences[self as Unit] = pf.closest_neighbors(cell, to)
		for i in range(0,followers.size()):
			preferences[followers[i] as Unit] = pf.closest_neighbors(followers[i].cell, targets[i])
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
					unmoved.erase(unit)
					successful_move = true
					# Maybe we've freed up space for an earlier unit.
					break
			if successful_move:
				choice = 0
			else:
				choice +=1
		await _await_movement()
		await get_tree().create_timer(0.05).timeout
	available = true
