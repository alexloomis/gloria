extends Node

# probably make static when large terrain backgrounds implemented
# class_name Terrain

var terrain: Dictionary[Vector2i, TILE]

#var tile_data: Array[PackedScene] = [preload("res://map/tile/obstacle.tscn"),
#preload("res://map/tile/grass.tscn"),
#preload("res://map/tile/water.tscn")]

enum TILE {OBSTACLE, GRASS, WATER}

var tile_data: Dictionary[TILE, PackedScene] = {TILE.OBSTACLE: preload("res://map/tile/obstacle.tscn"),
TILE.GRASS: preload("res://map/tile/grass.tscn"),
TILE.WATER: preload("res://map/tile/water.tscn"),}

func _init() -> void:
	for x in range(Grid.cols):
		for y in range(Grid.rows):
			var tile: TILE = TILE.OBSTACLE
			var rand: float = 100 * randf()
			if rand <= 95.0:
				tile = TILE.GRASS
			elif  rand <= 99.0:
				tile = TILE.WATER
			terrain[Vector2i(x,y)] = tile
