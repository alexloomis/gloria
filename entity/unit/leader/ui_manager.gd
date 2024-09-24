extends Node

class_name UIManager

var reach: Array[Vector2i]
var unit: Leader

func _on_selected() -> void:
	var costs: Dictionary = unit.pf.costs(unit.cell, unit.speed)
	for cell: Vector2i in costs:
		if costs[cell] <= unit.speed:
			reach.append(cell)

func _on_deselected() -> void:
	reach = []
