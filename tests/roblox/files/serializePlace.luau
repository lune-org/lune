local fs = require("@lune/fs")
local roblox = require("@lune/roblox")
local Instance = roblox.Instance

-- Smoke tests
do
	local game = Instance.new("DataModel")

	local workspace = game:GetService("Workspace")

	local model = Instance.new("Model")
	local part = Instance.new("Part")

	part.Parent = model
	model.Parent = workspace

	local placeAsBinary = roblox.serializePlace(game)
	local placeAsXml = roblox.serializePlace(game, true)

	fs.writeFile("bin/temp-place.rbxl", placeAsBinary)
	fs.writeFile("bin/temp-place.rbxlx", placeAsXml)

	local savedFileBinary = fs.readFile("bin/temp-place.rbxl")
	local savedFileXml = fs.readFile("bin/temp-place.rbxlx")

	local savedBinary = roblox.deserializePlace(savedFileBinary)
	local savedXml = roblox.deserializePlace(savedFileXml)

	assert(savedBinary.Name ~= "ROOT")
	assert(savedXml.Name ~= "ROOT")

	assert(savedBinary.ClassName == "DataModel")
	assert(savedXml.ClassName == "DataModel")
end

-- Ensure Ref properties are preserved across services
do
	local game = Instance.new("DataModel")
	local ReplicatedStorage = Instance.new("ReplicatedStorage")
	local Workspace = Instance.new("Workspace")

	Workspace.Parent = game
	ReplicatedStorage.Parent = game

	local part = Instance.new("Part")
	part.Parent = ReplicatedStorage

	local objectValue = Instance.new("ObjectValue")
	objectValue.Value = part
	objectValue.Parent = Workspace

	local serialized = roblox.serializePlace(game)
	local deserialized = roblox.deserializePlace(serialized)

	assert(deserialized.Workspace.ObjectValue.Value == deserialized.ReplicatedStorage.Part)
end
