local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local root = Instance.new("Model")
local child1 = Instance.new("Part")
local child2 = Instance.new("Part")

child1.Parent = root
child2.Parent = root

assert(#root:GetChildren() == 2)
assert(root:GetChildren()[1] == child1)
assert(root:GetChildren()[2] == child2)

root:ClearAllChildren()

assert(#root:GetChildren() == 0)

assert(not pcall(function()
	return child1.Name
end))

assert(not pcall(function()
	return child1.Parent
end))

assert(not pcall(function()
	return child2.Name
end))

assert(not pcall(function()
	return child2.Parent
end))
