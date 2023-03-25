local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local root = Instance.new("Folder")
local inner1 = Instance.new("Model")
local inner1_1 = Instance.new("Part")
local inner1_2 = Instance.new("Part")
local inner2 = Instance.new("Model")
local inner2_1 = Instance.new("Part")
local inner2_2 = Instance.new("Part")

inner1.Parent = root
inner1_1.Parent = inner1
inner1_2.Parent = inner1
inner2.Parent = root
inner2_1.Parent = inner2
inner2_2.Parent = inner2

assert(root:FindFirstDescendant("Folder") == nil)
assert(root:FindFirstDescendant("Model") == inner1)
assert(root:FindFirstDescendant("Part") == inner1_1)

inner1:Destroy()

assert(root:FindFirstDescendant("Folder") == nil)
assert(root:FindFirstDescendant("Model") == inner2)
assert(root:FindFirstDescendant("Part") == inner2_1)

inner2_1:Destroy()

assert(root:FindFirstDescendant("Part") == inner2_2)
