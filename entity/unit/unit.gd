extends Node2D

class_name Unit

signal finished_moving

@export var disp_name: StringName
@export var texture: Texture2D:
	set(val):
		texture = val
		if not sprite:
			await ready
		sprite.texture = val
		_resize_sprite()

var frame: Sprite2D
var leader: Leader
var path2d: Path2D
var pf2d: PathFollow2D
var sprite: Sprite2D

var cell: Vector2i:
	set(val):
		Roster.deregister(self)
		position = Grid.cell_to_px(val)
		Roster.register(self)
	get:
		return Grid.px_to_cell(position)

var moving: bool = false:
	set(val):
		moving = val
		if not moving:
			finished_moving.emit()

var selected: bool = false:
	set(val):
		selected = val
		if not frame:
			await ready
		frame.visible = val

func _ready() -> void:
	_init_path2d()
	_init_pf2d()
	_init_sprite()
	_init_frame()
	Roster.register(self)
#	move_adjacent(cell)

func _process(delta: float) -> void:
	pf2d.progress += leader.animation_speed * delta
	if pf2d.progress_ratio >= 1.0:
		pf2d.progress = 0
		pf2d.position = Vector2(0,0)
		path2d.curve.clear_points()
		set_process(false)
		moving = false

func die() -> void:
	Roster.deregister(self)
	queue_free()

#region Display
func _resize_sprite() -> void:
	sprite.scale = Grid.cell_size as Vector2 / sprite.texture.get_size()

func _init_sprite() -> void:
	sprite = Sprite2D.new()
	sprite.texture = texture
	pf2d.add_child(sprite)
	sprite.centered = false
	_resize_sprite()

func _init_frame() -> void:
	frame = Sprite2D.new()
	frame.texture = preload("res://textures_test/frame-2-orange.png")
	pf2d.add_child(frame)
	frame.centered = false
	frame.scale = Grid.cell_size as Vector2 / frame.texture.get_size()
	frame.visible = false
#endregion

#region Movement
func _init_path2d() -> void:
	path2d = Path2D.new()
	add_child(path2d)
	path2d.curve = Curve2D.new()

func _init_pf2d() -> void:
	pf2d = PathFollow2D.new()
	path2d.add_child(pf2d)
	pf2d.rotates = false
	pf2d.loop = false

func _push(path_px: PackedVector2Array) -> void:
	path2d.curve = Curve2D.new()
	for point in path_px:
		path2d.curve.add_point(point - position)
	# Sets position correctly before process tick, removing a flicker.
	pf2d.progress = 0
	moving = true
	set_process(true)

# Does not verify adjacency
func move_adjacent(to: Vector2i) -> void:
	if to == cell:
		return
	var path_px: PackedVector2Array = Grid.path_px(cell, to)
	cell = to
	_push(path_px)
#endregion
