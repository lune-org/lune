-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Axes = _G.Axes
local Enum = _G.Enum

-- Constructors & properties

Axes.new()
Axes.new(Enum.Axis.X)
Axes.new(Enum.Axis.X, Enum.NormalId.Top)

assert(not pcall(function()
	return Axes.new(false)
end))
assert(not pcall(function()
	return Axes.new({})
end))
assert(not pcall(function()
	return Axes.new(newproxy(true))
end))

assert(Axes.new().X == false)
assert(Axes.new().Y == false)
assert(Axes.new().Z == false)

assert(Axes.new(Enum.Axis.X, Enum.NormalId.Top).X == true)
assert(Axes.new(Enum.Axis.X, Enum.NormalId.Top).Y == true)
assert(Axes.new(Enum.Axis.X, Enum.NormalId.Top).Z == false)

assert(Axes.new(Enum.Axis.X, Enum.NormalId.Left).X == true)
assert(Axes.new(Enum.Axis.X, Enum.NormalId.Right).X == true)

assert(Axes.new(Enum.NormalId.Front, Enum.NormalId.Back).X == false)
assert(Axes.new(Enum.NormalId.Front, Enum.NormalId.Back).Y == false)
assert(Axes.new(Enum.NormalId.Front, Enum.NormalId.Back).Z == true)

-- Ops

assert(not pcall(function()
	return Axes.new() + Axes.new()
end))
assert(not pcall(function()
	return Axes.new() / Axes.new()
end))
