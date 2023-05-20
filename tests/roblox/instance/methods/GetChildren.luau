local fs = require("@lune/fs")
local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local modelFile = fs.readFile("tests/roblox/rbx-test-files/models/three-nested-folders/binary.rbxm")
local model = roblox.deserializeModel(modelFile)[1]

assert(#model:GetChildren() == 1)

local newChild = Instance.new("Model")
newChild.Parent = model

assert(#model:GetChildren() == 2)
assert(table.find(model:GetChildren(), newChild) ~= nil)

newChild:Destroy()

assert(#model:GetChildren() == 1)
assert(table.find(model:GetChildren(), newChild) == nil)
