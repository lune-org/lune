-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Vector3int16 = _G.Vector3int16

-- Constructors

Vector3int16.new()
Vector3int16.new(0)
Vector3int16.new(0, 0)
Vector3int16.new(0, 0, 0)

assert(not pcall(function()
	return Vector3int16.new(999_999, 999_999, 999_999)
end))
assert(not pcall(function()
	return Vector3int16.new(false)
end))
assert(not pcall(function()
	return Vector3int16.new("", "")
end))
assert(not pcall(function()
	return Vector3int16.new(newproxy(true))
end))

-- Ops

assert(Vector3int16.new(2, 4, 8) + Vector3int16.new(1, 1, 1) == Vector3int16.new(3, 5, 9))
assert(Vector3int16.new(2, 4, 8) - Vector3int16.new(1, 1, 1) == Vector3int16.new(1, 3, 7))
assert(Vector3int16.new(2, 4, 8) * Vector3int16.new(1, 1, 2) == Vector3int16.new(2, 4, 16))
assert(Vector3int16.new(2, 4, 8) / Vector3int16.new(1, 1, 2) == Vector3int16.new(2, 4, 4))

assert(Vector3int16.new(2, 4, 8) * 2 == Vector3int16.new(4, 8, 16))
assert(Vector3int16.new(2, 4, 8) / 2 == Vector3int16.new(1, 2, 4))
