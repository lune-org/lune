local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local root = Instance.new("Folder")
local child = Instance.new("Model")
local nested = Instance.new("Tool")
local descendant = Instance.new("Part")

descendant.Parent = nested
nested.Parent = child
child.Parent = root

assert(descendant:FindFirstAncestorOfClass("Part") == nil)
assert(descendant:FindFirstAncestorOfClass("Tool") == nested)
assert(descendant:FindFirstAncestorOfClass("Model") == child)
assert(descendant:FindFirstAncestorOfClass("Folder") == root)
