-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Region3 = _G.Region3
local Vector3 = _G.Vector3
local CFrame = _G.CFrame

local min = Vector3.new(-2, -2, -2)
local max = Vector3.new(2, 2, 2)

-- Constructors & properties

Region3.new(min, max)

assert(not pcall(function()
	return Region3.new(false)
end))
assert(not pcall(function()
	return Region3.new("", "")
end))
assert(not pcall(function()
	return Region3.new(newproxy(true))
end))

assert(Region3.new(min, max).CFrame == CFrame.new(0, 0, 0))
assert(Region3.new(min, max).Size == Vector3.new(4, 4, 4))

-- Ops

assert(not pcall(function()
	return Region3.new(min, max) + Region3.new(min, max)
end))
assert(not pcall(function()
	return Region3.new(min, max) / Region3.new(min, max)
end))

-- Methods

assert(Region3.new(min, max):ExpandToGrid(1) == Region3.new(min, max))

assert(
	Region3.new(min, max):ExpandToGrid(3)
		== Region3.new(Vector3.new(-3, -3, -3), Vector3.new(3, 3, 3))
)

assert(
	Region3.new(min, max):ExpandToGrid(4)
		== Region3.new(Vector3.new(-4, -4, -4), Vector3.new(4, 4, 4))
)

assert(
	Region3.new(min, max):ExpandToGrid(7.5)
		== Region3.new(Vector3.new(-7.5, -7.5, -7.5), Vector3.new(7.5, 7.5, 7.5))
)
