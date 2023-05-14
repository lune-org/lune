local fs = require("@lune/fs")
local roblox = require("@lune/roblox") :: any

local modelDirs = {}
for _, dirName in fs.readDir("tests/roblox/rbx-test-files/places") do
	table.insert(modelDirs, "tests/roblox/rbx-test-files/places/" .. dirName)
end

for _, modelDir in modelDirs do
	local modelBinary = roblox.readModelFile(modelDir .. "/binary.rbxl")
	local modelXml = roblox.readModelFile(modelDir .. "/xml.rbxlx")

	for _, modelInstance in modelBinary do
		assert(modelInstance:IsA("Instance"))
	end

	for _, modelInstance in modelXml do
		assert(modelInstance:IsA("Instance"))
	end
end
