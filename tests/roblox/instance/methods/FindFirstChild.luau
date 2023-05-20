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

assert(child:FindFirstChild("Folder") == nil)
assert(child:FindFirstChild("Model") == adjacent)
assert(child:FindFirstChild("Tool") == nested)
assert(child:FindFirstChild("Part") == nil)
