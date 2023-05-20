local fs = require("@lune/fs")
local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local modelFile = fs.readFile("tests/roblox/rbx-test-files/models/three-nested-folders/binary.rbxm")
local model = roblox.deserializeModel(modelFile)[1]

assert(#model:GetDescendants() == 2)

local newChild = Instance.new("Model")
newChild.Parent = model

assert(#model:GetDescendants() == 3)
assert(table.find(model:GetDescendants(), newChild) == 2)

newChild:Destroy()

assert(#model:GetDescendants() == 2)
assert(table.find(model:GetDescendants(), newChild) == nil)
