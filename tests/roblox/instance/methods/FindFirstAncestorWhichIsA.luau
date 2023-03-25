local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local root = Instance.new("Folder")
local child = Instance.new("Model")
local nested = Instance.new("Tool")
local descendant = Instance.new("Part")

descendant.Parent = nested
nested.Parent = child
child.Parent = root

assert(descendant:FindFirstAncestorWhichIsA("Part") == nil)
assert(descendant:FindFirstAncestorWhichIsA("Tool") == nested)
assert(descendant:FindFirstAncestorWhichIsA("Model") == nested)
assert(descendant:FindFirstAncestorWhichIsA("Folder") == root)
