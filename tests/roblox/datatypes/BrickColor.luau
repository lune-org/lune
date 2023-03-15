-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local BrickColor = _G.BrickColor
local Color3 = _G.Color3

-- Constructors & properties

BrickColor.new(1)
BrickColor.new("Medium stone grey")

assert(not pcall(function()
	return BrickColor.new(false)
end))
assert(not pcall(function()
	return BrickColor.new({})
end))
assert(not pcall(function()
	return BrickColor.new(newproxy(true))
end))

assert(BrickColor.new("Really red").R == 1)
assert(BrickColor.new("Really red").G == 0)
assert(BrickColor.new("Really red").B == 0)

assert(BrickColor.new("Really red").Number == 1004)
assert(BrickColor.new("Really red").Name == "Really red")
assert(BrickColor.new("Really red").Color == Color3.new(1, 0, 0))

-- Ops

assert(not pcall(function()
	return BrickColor.new(1) + BrickColor.new(2)
end))
assert(not pcall(function()
	return BrickColor.new(1) / BrickColor.new(2)
end))
