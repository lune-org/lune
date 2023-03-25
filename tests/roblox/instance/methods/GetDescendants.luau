local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local model =
	roblox.readModelFile("tests/roblox/rbx-test-files/models/three-nested-folders/binary.rbxm")[1]

assert(#model:GetDescendants() == 2)

local newChild = Instance.new("Model")
newChild.Parent = model

assert(#model:GetDescendants() == 3)
assert(table.find(model:GetDescendants(), newChild) == 2)

newChild:Destroy()

assert(#model:GetDescendants() == 2)
assert(table.find(model:GetDescendants(), newChild) == nil)
