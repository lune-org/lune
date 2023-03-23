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

local savedBinary = roblox.readPlaceFile("bin/temp-place.rbxl")
local savedXml = roblox.readPlaceFile("bin/temp-place.rbxlx")

assert(savedBinary.Name ~= "ROOT")
assert(savedXml.Name ~= "ROOT")

assert(savedBinary.ClassName == "DataModel")
assert(savedXml.ClassName == "DataModel")
