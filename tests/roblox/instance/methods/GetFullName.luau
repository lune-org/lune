local fs = require("@lune/fs")
local roblox = require("@lune/roblox") :: any

local modelFile = fs.readFile("tests/roblox/rbx-test-files/models/three-nested-folders/binary.rbxm")
local model = roblox.deserializeModel(modelFile)[1]

local child = model:FindFirstChild("Parent")
local descendant = child:FindFirstChild("Child")

assert(descendant:GetFullName() == "Grandparent.Parent.Child")
assert(child:GetFullName() == "Grandparent.Parent")
assert(model:GetFullName() == "Grandparent")
