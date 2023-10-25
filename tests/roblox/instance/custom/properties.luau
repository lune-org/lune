local roblox = require("@lune/roblox")

local inst = roblox.Instance.new("Instance") :: any
local part = roblox.Instance.new("Part") :: any

-- Basic sanity checks for callbacks

local success = pcall(function()
	local _ = inst.Wat
end)
assert(not success, "Nonexistent properties should error")

roblox.implementProperty("Instance", "Wat", function()
	return nil
end)

local success2 = pcall(function()
	local _ = inst.Wat
end)
assert(success2, "Nonexistent properties should error, unless implemented")

-- Instance should be passed to callback

roblox.implementProperty("Instance", "PassingInstanceTest", function(instance)
	assert(instance == inst, "Invalid instance was passed to callback")
	return nil
end)
roblox.implementProperty("Part", "PassingPartTest", function(instance)
	assert(instance == part, "Invalid instance was passed to callback")
	return nil
end)
local _ = inst.PassingInstanceTest
local _ = part.PassingPartTest

-- Any number of args passed & returned should work

local counters = {}
roblox.implementProperty("Instance", "Counter", function(instance)
	local value = counters[instance:GetDebugId()] or 0
	value += 1
	counters[instance:GetDebugId()] = value
	return value
end, function(instance, value)
	counters[instance:GetDebugId()] = value
end)

assert(inst.Counter == 1, "implementProperty callback should return proper values")
assert(inst.Counter == 2, "implementProperty callback should return proper values")
assert(inst.Counter == 3, "implementProperty callback should return proper values")

inst.Counter = 10

assert(inst.Counter == 11, "implementProperty callback should set proper values")
assert(inst.Counter == 12, "implementProperty callback should return proper values")
assert(inst.Counter == 13, "implementProperty callback should return proper values")

-- Properties implemented by Lune should take precedence

roblox.implementProperty("Instance", "Parent", function()
	error("unreachable")
end)
local _ = inst.Parent
local _ = part.Parent
