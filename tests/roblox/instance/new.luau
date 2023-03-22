local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

-- Should not allow creating unknown classes
assert(not pcall(function()
	Instance.new("asdf")
end))

-- Should be case sensitive
assert(not pcall(function()
	Instance.new("part")
end))

-- Should allow "not creatable" tagged classes to be created
Instance.new("BasePart")

-- Should have correct classnames
assert(Instance.new("Part").ClassName == "Part")
assert(Instance.new("Folder").ClassName == "Folder")
assert(Instance.new("ReplicatedStorage").ClassName == "ReplicatedStorage")

-- Should have initial names that are the same as the class name
assert(Instance.new("Part").Name == "Part")
assert(Instance.new("Folder").Name == "Folder")
assert(Instance.new("ReplicatedStorage").Name == "ReplicatedStorage")

-- Parent should be nil until parented
local folder = Instance.new("Folder")
local model = Instance.new("Model")
assert(folder.Parent == nil)
assert(model.Parent == nil)

-- Parenting and indexing should work
model.Parent = folder
assert(model.Parent == folder)
assert(folder.Model == model)

-- Parenting to nil should work
model.Parent = nil
assert(model.Parent == nil)

-- Name should be able to be set, and should not be nillable
model.Name = "MyCoolModel"
assert(model.Name == "MyCoolModel")
assert(not pcall(function()
	model.Name = nil
end))
assert(model.Name == "MyCoolModel")
