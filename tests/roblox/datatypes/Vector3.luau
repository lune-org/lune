local roblox = require("@lune/roblox") :: any
local Vector3 = roblox.Vector3

-- Constructors & properties

Vector3.new()
Vector3.new(0)
Vector3.new(0, 0)
Vector3.new(0, 0, 0)
Vector3.new(0 / 0, 0 / 0)
Vector3.new(0 / 0, 0 / 0, 0 / 0)

assert(not pcall(function()
	return Vector3.new(false)
end))
assert(not pcall(function()
	return Vector3.new("", "")
end))
assert(not pcall(function()
	return Vector3.new(newproxy(true))
end))

assert(Vector3.new(1, 2, 3).X == 1)
assert(Vector3.new(1, 2, 3).Y == 2)
assert(Vector3.new(1, 2, 3).Z == 3)

-- Constants

assert(Vector3.one == Vector3.new(1, 1, 1))
assert(Vector3.zero == Vector3.new(0, 0, 0))
assert(Vector3.xAxis == Vector3.new(1, 0, 0))
assert(Vector3.yAxis == Vector3.new(0, 1, 0))
assert(Vector3.zAxis == Vector3.new(0, 0, 1))

-- Ops

assert(Vector3.new(2, 4, 8) + Vector3.new(1, 1, 1) == Vector3.new(3, 5, 9))
assert(Vector3.new(2, 4, 8) - Vector3.new(1, 1, 1) == Vector3.new(1, 3, 7))
assert(Vector3.new(2, 4, 8) * Vector3.new(1, 1, 2) == Vector3.new(2, 4, 16))
assert(Vector3.new(2, 4, 8) / Vector3.new(1, 1, 2) == Vector3.new(2, 4, 4))

assert(Vector3.new(2, 4, 8) * 2 == Vector3.new(4, 8, 16))
assert(Vector3.new(2, 4, 8) / 2 == Vector3.new(1, 2, 4))

-- TODO: Vector math
