local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local model = Instance.new("Model")
local part = Instance.new("Part")
part.Parent = model

local TAG_NAME = "InstanceTagName"

assert(model:HasTag(TAG_NAME) == false)
assert(part:HasTag(TAG_NAME) == false)

part:AddTag(TAG_NAME)

assert(model:HasTag(TAG_NAME) == false)
assert(part:HasTag(TAG_NAME) == true)

part:RemoveTag(TAG_NAME)

assert(model:HasTag(TAG_NAME) == false)
assert(part:HasTag(TAG_NAME) == false)

assert(#model:GetTags() == 0)
assert(#part:GetTags() == 0)

model:AddTag(TAG_NAME)
part:AddTag(TAG_NAME)

assert(#model:GetTags() == 1)
assert(#part:GetTags() == 1)
assert(model:GetTags()[1] == TAG_NAME)
assert(part:GetTags()[1] == TAG_NAME)
