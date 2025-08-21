extends Unit

class_name Leader

@export_range(0,10) var speed: int
@export var formation: Shape
@export var follower_data: Dictionary[PackedScene, int]
@export var terrain_data: Dictionary[Terrain.TILE, int]

var animation_speed: int:
	get:
		return (Settings.animation_speed as float) * sqrt(speed as float) as int
var available: bool
var followers: Array[Follower]:
	set(val):
		followers = val
		formation.size = followers.size() + 1
		print("updated followers")
var nav: Navigator
var pf: Pathfinder

func _ready() -> void:
	super()
	_init_pf()
	_init_nav()
	_place_followers()
	available = true
	leader = self

func _init_pf() -> void:
	pf = Pathfinder.new()
	pf._terrain_data = terrain_data
	pf.update()

func _init_nav() -> void:
	nav = Navigator.new()

func _instantiate_followers() -> void:
	followers.clear()
	for scene in follower_data:
		for _i in follower_data[scene]:
			var new_follower: Follower = scene.instantiate() as Follower
			new_follower.leader = self
			followers += [new_follower]

func _place_followers() -> void:
	if not formation:
		return
	_instantiate_followers()
	var idx: int = 0
	print(formation.size)
	print(formation.tiles)
	print(formation.base_tiles)
	for follower in followers:
		var placed: bool = false
		while not placed:
			var candidate: Vector2i = cell + formation.tiles[idx]
			if Grid.in_bounds(candidate) and pf.is_clear(candidate):
				follower.cell = candidate
				placed = true
			idx += 1
	for follower in followers:
		add_sibling.call_deferred(follower)

func _trim_paths(paths: Array[Array]) -> void:
	while paths[0].size() >= 2:
		var stationary: bool = true
		for path in paths:
			if not path[-1] == path[-2]:
				stationary = false
				break
		if stationary:
			for path in paths:
				path.pop_back()
		else:
			return

func get_paths(to: Vector2i) -> Array[Array]:
	nav.reset(self)
	var paths: Array[Array] = nav.find_nonempty_paths(to)
	_trim_paths(paths)
	return paths

func _move_adjacent_by_idx(idx: int, to: Vector2i) -> void:
	if idx == 0:
		move_adjacent(to)
	else:
		followers[idx-1].move_adjacent(to)

func move_formation(to: Vector2i) -> void:
	if not available:
		return
	available = false
	var paths: Array[Array] = get_paths(to)
	print(paths[0])
	for time in paths[0].size():
		for idx in paths.size():
			if time < paths[idx].size():
				var new_cell: Vector2i = paths[idx][time]
				_move_adjacent_by_idx(idx, new_cell)
		await get_tree().create_timer(0.3).timeout
	available = true
