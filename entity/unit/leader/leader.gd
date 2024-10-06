extends Unit

class_name Leader

@export_range(0,10) var speed: int
@export var formation: Formation
@export var follower_data: Dictionary[PackedScene, int]
@export var terrain_data: Dictionary[Terrain.TILE, int]

var animation_speed: int:
	get:
		return (Settings.animation_speed as float) * sqrt(speed as float) as int
var available: bool
var followers: Array[Follower]
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
	#nav.reset(self)

func _instantiate_followers() -> void:
	followers.clear()
	for scene in follower_data:
		for _i in follower_data[scene]:
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
			if Grid.in_bounds(candidate) and pf.is_clear(candidate):
				follower.cell = candidate
				placed = true
			idx += 1
	for follower in followers:
		add_sibling.call_deferred(follower)

func get_paths(to: Vector2i) -> Array[Array]:
	nav.reset(self)
	return nav.find_paths(to)

func move_formation(to: Vector2i) -> void:
	if not available:
		return
	available = false
	var paths: Array[Array] = get_paths(to)
	for time in speed:
		var done: bool = true
		if paths[0].size() > 1:
			var coord: Vector3i = paths[0][0]
			if time == coord.z:
				paths[0].remove_at(0)
				var new_coord: Vector3i = paths[0][0]
				move_adjacent(Util.project(new_coord))
				@warning_ignore("unsafe_call_argument")
				if not Util.project(paths[0][-1]) == cell:
					done = false
		for idx in range(1, followers.size() + 1):
			if paths[idx].size() > 1:
				var coord: Vector3i = paths[idx][0]
				if time == coord.z:
					paths[idx].remove_at(0)
					var new_coord: Vector3i = paths[idx][0]
					followers[idx-1].move_adjacent(Util.project(new_coord))
					@warning_ignore("unsafe_call_argument")
					if not Util.project(paths[idx][-1]) == followers[idx-1].cell:
						done = false
		if not done:
			await get_tree().create_timer(0.3).timeout
	available = true
