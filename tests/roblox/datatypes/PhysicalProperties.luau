local roblox = require("@lune/roblox") :: any
local PhysicalProperties = roblox.PhysicalProperties
local Enum = roblox.Enum

-- Constructors & properties

PhysicalProperties.new(Enum.Material.SmoothPlastic)
PhysicalProperties.new(0, 0, 0)
PhysicalProperties.new(0, 0, 0, 0, 0)

assert(not pcall(function()
	return PhysicalProperties.new()
end))
assert(not pcall(function()
	return PhysicalProperties.new(false)
end))
assert(not pcall(function()
	return PhysicalProperties.new({})
end))
assert(not pcall(function()
	return PhysicalProperties.new(newproxy(true))
end))
assert(not pcall(function()
	return PhysicalProperties.new(Enum.Axis.X)
end))

assert(PhysicalProperties.new(1, 2, 3).FrictionWeight == 1)
assert(PhysicalProperties.new(1, 2, 3).ElasticityWeight == 1)
assert(PhysicalProperties.new(1, 2, 3).AcousticAbsorption == 1)

assert(PhysicalProperties.new(1, 2, 3, 4, 5, 6).Density == 1)
assert(PhysicalProperties.new(1, 2, 3, 4, 5, 6).Friction == 2)
assert(PhysicalProperties.new(1, 2, 3, 4, 5, 6).Elasticity == 3)
assert(PhysicalProperties.new(1, 2, 3, 4, 5, 6).FrictionWeight == 4)
assert(PhysicalProperties.new(1, 2, 3, 4, 5, 6).ElasticityWeight == 5)
assert(PhysicalProperties.new(1, 2, 3, 4, 5, 6).AcousticAbsorption == 6)

local function fuzzyEq(n0: number, n1: number)
	return math.abs(n1 - n0) <= 0.0001
end

local plastic = PhysicalProperties.new(Enum.Material.Plastic)
assert(fuzzyEq(plastic.Density, 0.7))
assert(fuzzyEq(plastic.Friction, 0.3))
assert(fuzzyEq(plastic.Elasticity, 0.5))

local splastic = PhysicalProperties.new(Enum.Material.SmoothPlastic)
assert(fuzzyEq(splastic.Density, 0.7))
assert(fuzzyEq(splastic.Friction, 0.2))
assert(fuzzyEq(splastic.Elasticity, 0.5))

local sand = PhysicalProperties.new(Enum.Material.Sand)
assert(fuzzyEq(sand.Density, 1.6))
assert(fuzzyEq(sand.Friction, 0.5))
assert(fuzzyEq(sand.Elasticity, 0.05))
assert(fuzzyEq(sand.FrictionWeight, 5))
assert(fuzzyEq(sand.ElasticityWeight, 2.5))
assert(fuzzyEq(sand.AcousticAbsorption, 0.55))

-- Ops

assert(not pcall(function()
	return PhysicalProperties.new() + PhysicalProperties.new()
end))
assert(not pcall(function()
	return PhysicalProperties.new() / PhysicalProperties.new()
end))
