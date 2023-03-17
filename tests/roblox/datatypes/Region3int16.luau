-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Region3int16 = _G.Region3int16
local Vector3int16 = _G.Vector3int16
local Vector3 = _G.Vector3

local min = Vector3int16.new(0, 0, 0)
local max = Vector3int16.new(2, 2, 2)

-- Constructors & properties

Region3int16.new(min, max)

assert(not pcall(function()
	return Region3int16.new(false)
end))
assert(not pcall(function()
	return Region3int16.new("", "")
end))
assert(not pcall(function()
	return Region3int16.new(newproxy(true))
end))
assert(not pcall(function()
	return Region3int16.new(Vector3.new(), Vector3.new())
end))

assert(Region3int16.new(min, max).Min == min)
assert(Region3int16.new(min, max).Max == max)

-- Ops

assert(not pcall(function()
	return Region3int16.new(min, max) + Region3int16.new(min, max)
end))
assert(not pcall(function()
	return Region3int16.new(min, max) / Region3int16.new(min, max)
end))
