-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Ray = _G.Ray
local Vector3 = _G.Vector3

local origin = Vector3.zero
local direction = Vector3.zAxis * 10

-- Constructors & properties

Ray.new(origin, direction)

assert(not pcall(function()
	return Ray.new(false)
end))
assert(not pcall(function()
	return Ray.new("", "")
end))
assert(not pcall(function()
	return Ray.new(newproxy(true))
end))

assert(Ray.new(origin, direction).Origin == origin)
assert(Ray.new(origin, direction).Direction == direction)

assert(Ray.new(origin, direction).Unit.Origin == origin)
assert(Ray.new(origin, direction).Unit.Direction == direction.Unit)

-- Ops

assert(not pcall(function()
	return Ray.new(origin, direction) + Ray.new(origin, direction)
end))
assert(not pcall(function()
	return Ray.new(origin, direction) / Ray.new(origin, direction)
end))

-- Methods

assert(Ray.new(origin, direction):ClosestPoint(origin) == origin)
assert(Ray.new(origin, direction):Distance(origin) == 0)

for z = 0, 10, 1 do
	local x = if z % 2 == 0 then 2.5 else 7.5
	assert(
		Ray.new(origin, direction):ClosestPoint(Vector3.new(x, 0, z))
			== Vector3.zero + Vector3.zAxis * z
	)
	assert(Ray.new(origin, direction):Distance(Vector3.new(x, 0, z)) == x)
end
