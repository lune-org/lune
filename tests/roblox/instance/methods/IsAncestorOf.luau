local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local root = Instance.new("Folder")
local child = Instance.new("Model")
local descendant = Instance.new("Part")

child.Parent = root
descendant.Parent = child

assert(not root:IsAncestorOf(root))
assert(not child:IsAncestorOf(root))
assert(not descendant:IsAncestorOf(root))

assert(root:IsAncestorOf(child))
assert(not child:IsAncestorOf(child))
assert(not descendant:IsAncestorOf(child))

assert(root:IsAncestorOf(descendant))
assert(child:IsAncestorOf(descendant))
assert(not descendant:IsAncestorOf(descendant))
