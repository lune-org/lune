local fs = require("@lune/fs")
local roblox = require("@lune/roblox")
local Instance = roblox.Instance

-- Smoke tests
do
	local instances = {
		Instance.new("Model"),
		Instance.new("Part"),
	}

	local modelAsBinary = roblox.serializeModel(instances)
	local modelAsXml = roblox.serializeModel(instances, true)

	fs.writeFile("bin/temp-model.rbxm", modelAsBinary)
	fs.writeFile("bin/temp-model.rbxmx", modelAsXml)

	local savedFileBinary = fs.readFile("bin/temp-model.rbxm")
	local savedFileXml = fs.readFile("bin/temp-model.rbxmx")

	local savedBinary = roblox.deserializeModel(savedFileBinary)
	local savedXml = roblox.deserializeModel(savedFileXml)

	assert(savedBinary[1].Name ~= "ROOT")
	assert(savedXml[1].Name ~= "ROOT")

	assert(savedBinary[1].Name ~= "DataModel")
	assert(savedXml[1].Name ~= "DataModel")

	assert(savedBinary[1].ClassName == "Model")
	assert(savedBinary[2].ClassName == "Part")

	assert(savedXml[1].ClassName == "Model")
	assert(savedXml[2].ClassName == "Part")
end

-- Ensure Ref properties are preserved across descendants of multi-root model siblings
do
	local part = Instance.new("Part")

	local particleEmitter = Instance.new("ParticleEmitter")
	particleEmitter.Parent = part

	local folder = Instance.new("Folder")

	local objectValue = Instance.new("ObjectValue")
	objectValue.Value = particleEmitter
	objectValue.Parent = folder

	local serialized = roblox.serializeModel({ part, folder })
	local deserialized = roblox.deserializeModel(serialized)

	assert(deserialized[2].ObjectValue.Value == deserialized[1].ParticleEmitter)
end
