local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local model =
	roblox.readModelFile("tests/roblox/rbx-test-files/models/three-nested-folders/binary.rbxm")[1]

assert(#model:GetChildren() == 1)

local newChild = Instance.new("Model")
newChild.Parent = model

assert(#model:GetChildren() == 2)
assert(table.find(model:GetChildren(), newChild) ~= nil)

newChild:Destroy()

assert(#model:GetChildren() == 1)
assert(table.find(model:GetChildren(), newChild) == nil)
