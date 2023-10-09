local roblox = require("@lune/roblox")

local inst = roblox.Instance.new("Instance") :: any
local part = roblox.Instance.new("Part") :: any

-- Basic sanity checks for callbacks

local success = pcall(function()
	inst:Wat()
end)
assert(not success, "Nonexistent methods should error")

roblox.implementMethod("Instance", "Wat", function() end)

local success2 = pcall(function()
	inst:Wat()
end)
assert(success2, "Nonexistent methods should error, unless implemented")

-- Instance should be passed to callback

roblox.implementMethod("Instance", "PassingInstanceTest", function(instance)
	assert(instance == inst, "Invalid instance was passed to callback")
end)
roblox.implementMethod("Part", "PassingPartTest", function(instance)
	assert(instance == part, "Invalid instance was passed to callback")
end)
inst:PassingInstanceTest()
part:PassingPartTest()

-- Any number of args passed & returned should work

roblox.implementMethod("Instance", "Echo", function(_, ...)
	return ...
end)

local one, two, three = inst:Echo("one", "two", "three")
assert(one == "one", "implementMethod callback should return proper values")
assert(two == "two", "implementMethod callback should return proper values")
assert(three == "three", "implementMethod callback should return proper values")

-- Methods implemented by Lune should take precedence

roblox.implementMethod("Instance", "FindFirstChild", function()
	error("unreachable")
end)
inst:FindFirstChild("Test")
part:FindFirstChild("Test")
