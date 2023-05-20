local fs = require("@lune/fs")
local roblox = require("@lune/roblox")

local modelFile = fs.readFile("tests/roblox/rbx-test-files/models/three-nested-folders/binary.rbxm")
local model = roblox.deserializeModel(modelFile)[1]

local child = model:FindFirstChild("Parent")
assert(child ~= nil)
local descendant = child:FindFirstChild("Child")
assert(descendant ~= nil)

assert(descendant:GetFullName() == "Grandparent.Parent.Child")
assert(child:GetFullName() == "Grandparent.Parent")
assert(model:GetFullName() == "Grandparent")
