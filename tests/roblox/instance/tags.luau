local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local game = Instance.new("DataModel")
local cs = game:GetService("CollectionService")

local model = Instance.new("Model")
local part = Instance.new("Part")
part.Parent = model

local TAG_NAME = "InstanceTagName"

assert(cs:HasTag(model, TAG_NAME) == false)
assert(cs:HasTag(part, TAG_NAME) == false)

cs:AddTag(part, TAG_NAME)

assert(cs:HasTag(model, TAG_NAME) == false)
assert(cs:HasTag(part, TAG_NAME) == true)

cs:RemoveTag(part, TAG_NAME)

assert(cs:HasTag(model, TAG_NAME) == false)
assert(cs:HasTag(part, TAG_NAME) == false)

assert(#cs:GetTags(model) == 0)
assert(#cs:GetTags(part) == 0)

cs:AddTag(model, TAG_NAME)
cs:AddTag(part, TAG_NAME)

assert(#cs:GetTags(model) == 1)
assert(#cs:GetTags(part) == 1)
assert(cs:GetTags(model)[1] == TAG_NAME)
assert(cs:GetTags(part)[1] == TAG_NAME)
