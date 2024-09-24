extends Node

signal selection_changed(entity: Unit)

@onready var camera: Camera2D = %Camera
# @onready var move: MovementController = %MovementController
@onready var player: Node = %Player
@onready var tiles: Node = %Tiles

# UI state
# enum HOVERING {NONE, ENTITY, ABILITY}
enum SELECTED {NONE, ENTITY, ABILITY}

var selected_entity: Unit:
	set(v):
		selected_entity = v
		selection_changed.emit(selected_entity)
var state := SELECTED.NONE

func _ready() -> void:
	_init_tiles()

func _init_tiles() -> void:
	for x in range(Grid.cols):
		for y in range(Grid.rows):
			var coord := Vector2i(x,y)
			var type: int = Terrain.terrain[coord]
			var new_tile: Tile = Terrain.tile_data[type].instantiate()
			new_tile.position = Grid.cell_to_px(coord)
			tiles.add_child(new_tile)

func _manage_l_click() -> void:
	var cell: Vector2i = Grid.px_to_cell(camera.get_global_mouse_position())
	var target: Unit
	if Roster.roster.has(cell):
		target = Roster.roster[cell]
	match state:
		SELECTED.NONE:
			if target != null:
				select_entity(target)
		SELECTED.ENTITY:
			if selected_entity is Leader:
				var leader: Leader = selected_entity
				# leader.move(leader.cell, cell)
				leader.move_formation(cell)
#			if target != null:
#				select_entity(target)
#			else:
				# placeholder value
				#move.move(grid.px_to_cell(selected_entity.position), cell, 999999)
				#deselect_entity()
		SELECTED.ABILITY:
			pass

func _manage_r_click() -> void:
	match state:
		SELECTED.NONE:
			pass
		SELECTED.ENTITY:
			deselect_entity()
		SELECTED.ABILITY:
			pass

func _on_area_2d_input_event(_viewport: Viewport, event: InputEvent, _shape_idx: int) -> void:
	if event is InputEventMouseButton:
		if event.is_action_released("l_click"):
			_manage_l_click()
		if event.is_action_released("r_click"):
			_manage_r_click()

func deselect_entity() -> void:
	# destinations = []
	if selected_entity != null:
		selected_entity.selected = false
	selected_entity = null
	state = SELECTED.NONE
	# unhighlight_tiles()

func select_entity(entity: Unit) -> void:
	deselect_entity()
	state = SELECTED.ENTITY
	selected_entity = entity
	entity.selected = true
#	if entity.move_available():
#		destinations = move.destinations(grid.px_to_cell(entity.position), entity.data.move)
