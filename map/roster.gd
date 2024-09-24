extends Node

# class_name Roster

signal registered(entity: Unit)
signal deregistered(from: Vector2i)

var roster: Dictionary[Vector2i, Unit] = {}

func register(entity: Unit) -> void:
	if not entity.get_parent():
		return
	if not roster.has(entity.cell):
		roster[entity.cell] = entity
		registered.emit(entity)
	else:
		printerr("Cannot register ", entity.disp_name, " at cell ", entity.cell, ", cell already registered.")
		for cell: Vector2i in roster:
			print(entity.disp_name, " registered at ", cell)
		assert(false)

func deregister(entity: Unit) -> void:
	if not entity.get_parent():
		return
	roster.erase(entity.cell)
	deregistered.emit(entity.cell)
