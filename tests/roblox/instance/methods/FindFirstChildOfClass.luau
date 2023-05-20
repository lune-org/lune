local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local root = Instance.new("Folder")
local child = Instance.new("Model")
local nested = Instance.new("Tool")
local adjacent = Instance.new("Model")
local descendant = Instance.new("Part")

descendant.Parent = nested
nested.Parent = child
adjacent.Parent = child
child.Parent = root

assert(child:FindFirstChildOfClass("Folder") == nil)
assert(child:FindFirstChildOfClass("Tool") == nested)
assert(child:FindFirstChildOfClass("Model") == adjacent)
assert(child:FindFirstChildOfClass("Part") == nil)
