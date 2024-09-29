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
	pf.update()

func _init_nav() -> void:
	nav = Navigator.new()
	nav.reset(self)

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
			if Grid.in_bounds(candidate) and pf.is_clear(candidate):
				follower.cell = candidate
				placed = true
			idx += 1
	for follower in followers:
		add_sibling.call_deferred(follower)

func move_formation(to: Vector2i) -> void:
	nav.move_formation(to)
