local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local game = Instance.new("DataModel")

local workspace = game:GetService("Workspace")

local model = Instance.new("Model")
local part = Instance.new("Part")

part.Parent = model
model.Parent = workspace

roblox.writePlaceFile("bin/temp-place.rbxl", game)
roblox.writePlaceFile("bin/temp-place.rbxlx", game)
