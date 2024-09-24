extends Camera2D

var threshold: int = 10
var step: int = 10
@onready var inv_zoom: Vector2 = Vector2(1/zoom.x, 1/zoom.y):
	set(z):
		inv_zoom = z
		zoom.x = 1/z.x
		zoom.y = 1/z.y

func _process(_delta: float) -> void:
	var window: Window = get_window()
	var viewport_size: Vector2 = Vector2(window.size.x / zoom.x, window.size.y / zoom.y)
	var local_mouse_pos: Vector2i = get_local_mouse_position()
	if local_mouse_pos.x < threshold * inv_zoom.x:
		position.x -= step * inv_zoom.x
	elif local_mouse_pos.x > viewport_size.x - threshold * inv_zoom.x:
		position.x += step * inv_zoom.x
	if local_mouse_pos.y < threshold * inv_zoom.y:
		position.y -= step * inv_zoom.y
	elif local_mouse_pos.y > viewport_size.y - threshold * inv_zoom.y:
		position.y += step * inv_zoom.y

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("zoom_in"):
		if inv_zoom.x > 1:
			inv_zoom *= 9.0/10.0
	if event.is_action_pressed("zoom_out"):
		if inv_zoom.x < 5.5:
			inv_zoom *= 10.0/9.0
