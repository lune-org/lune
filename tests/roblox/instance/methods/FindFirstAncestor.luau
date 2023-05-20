local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local root = Instance.new("Folder")
local child = Instance.new("Model")
local nested = Instance.new("Tool")
local descendant = Instance.new("Part")

descendant.Parent = nested
nested.Parent = child
child.Parent = root

assert(descendant:FindFirstAncestor("Part") == nil)
assert(descendant:FindFirstAncestor("Tool") == nested)
assert(descendant:FindFirstAncestor("Model") == child)
assert(descendant:FindFirstAncestor("Folder") == root)
