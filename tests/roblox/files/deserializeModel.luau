local fs = require("@lune/fs")
local roblox = require("@lune/roblox")

local modelDirs = {}
for _, dirName in fs.readDir("tests/roblox/rbx-test-files/places") do
	table.insert(modelDirs, "tests/roblox/rbx-test-files/places/" .. dirName)
end

for _, modelDir in modelDirs do
	local modelFileBinary = fs.readFile(modelDir .. "/binary.rbxl")
	local modelFileXml = fs.readFile(modelDir .. "/xml.rbxlx")

	local modelBinary = roblox.deserializeModel(modelFileBinary)
	local modelXml = roblox.deserializeModel(modelFileXml)

	for _, modelInstance in modelBinary do
		assert(modelInstance:IsA("Instance"))
	end

	for _, modelInstance in modelXml do
		assert(modelInstance:IsA("Instance"))
	end
end
