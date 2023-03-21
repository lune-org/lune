local roblox = require("@lune/roblox") :: any
local Vector2int16 = roblox.Vector2int16

-- Constructors & properties

Vector2int16.new()
Vector2int16.new(0)
Vector2int16.new(0, 0)

assert(not pcall(function()
	return Vector2int16.new(999_999, 999_999)
end))
assert(not pcall(function()
	return Vector2int16.new(false)
end))
assert(not pcall(function()
	return Vector2int16.new("", "")
end))
assert(not pcall(function()
	return Vector2int16.new(newproxy(true))
end))

assert(Vector2int16.new(1, 2).X == 1)
assert(Vector2int16.new(1, 2).Y == 2)

-- Ops

assert(Vector2int16.new(2, 4) + Vector2int16.new(1, 1) == Vector2int16.new(3, 5))
assert(Vector2int16.new(2, 4) - Vector2int16.new(1, 1) == Vector2int16.new(1, 3))
assert(Vector2int16.new(2, 4) * Vector2int16.new(1, 2) == Vector2int16.new(2, 8))
assert(Vector2int16.new(2, 4) / Vector2int16.new(1, 2) == Vector2int16.new(2, 2))

assert(Vector2int16.new(2, 4) * 2 == Vector2int16.new(4, 8))
assert(Vector2int16.new(2, 4) / 2 == Vector2int16.new(1, 2))
