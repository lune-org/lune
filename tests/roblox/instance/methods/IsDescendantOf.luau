local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local root = Instance.new("Folder")
local child = Instance.new("Model")
local descendant = Instance.new("Part")

child.Parent = root
descendant.Parent = child

assert(not root:IsDescendantOf(root))
assert(child:IsDescendantOf(root))
assert(descendant:IsDescendantOf(root))

assert(not root:IsDescendantOf(child))
assert(not child:IsDescendantOf(child))
assert(descendant:IsDescendantOf(child))

assert(not root:IsDescendantOf(descendant))
assert(not child:IsDescendantOf(descendant))
assert(not descendant:IsDescendantOf(descendant))
