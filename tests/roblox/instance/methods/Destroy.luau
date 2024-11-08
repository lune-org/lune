local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local root = Instance.new("Folder")
local child = Instance.new("Model")
local descendant = Instance.new("Part")

child.Parent = root
descendant.Parent = child

root:Destroy()

assert(not pcall(function()
	return root.Name
end))

assert(not root.Parent)

assert(not pcall(function()
	return child.Name
end))

assert(not child.Parent)

assert(not pcall(function()
	return descendant.Name
end))

assert(not descendant.Parent)
