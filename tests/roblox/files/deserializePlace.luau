local fs = require("@lune/fs")
local roblox = require("@lune/roblox") :: any

local placeDirs = {}
for _, dirName in fs.readDir("tests/roblox/rbx-test-files/places") do
	table.insert(placeDirs, "tests/roblox/rbx-test-files/places/" .. dirName)
end

for _, placeDir in placeDirs do
	local placeFileBinary = fs.readFile(placeDir .. "/binary.rbxl")
	local placeFileXml = fs.readFile(placeDir .. "/xml.rbxlx")

	local placeBinary = roblox.deserializePlace(placeFileBinary)
	local placeXml = roblox.deserializePlace(placeFileXml)

	assert(placeBinary.ClassName == "DataModel")
	assert(placeXml.ClassName == "DataModel")

	assert(placeBinary:IsA("ServiceProvider"))
	assert(placeXml:IsA("ServiceProvider"))
end
