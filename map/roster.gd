extends Node

# class_name Roster

signal registered(at: Vector2i)
signal deregistered(from: Vector2i)

var roster: Dictionary[Vector2i, Unit] = {}

func register(entity: Unit) -> void:
	if not entity.get_parent():
		return
	var loc: Vector2i = entity.cell
	if not roster.has(loc):
		roster[loc] = entity
		registered.emit(loc)
	else:
		printerr("Cannot register ", entity.disp_name, " at cell ", entity.cell, ", cell already registered.")
		for cell: Vector2i in roster:
			print(entity.disp_name, " registered at ", cell)
		assert(false)

func deregister(entity: Unit) -> void:
	if not entity.get_parent():
		return
	var loc: Vector2i = entity.cell
	roster.erase(loc)
	deregistered.emit(loc)
