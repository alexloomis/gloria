extends Resource

class_name CellInfo

# Blocks for all times
var blocked: bool
# What cells is my own company blocking at any given time?
var block_times: Array[int]
var move_cost: int = Terrain.IMPASSIBLE
