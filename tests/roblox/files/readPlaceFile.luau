local fs = require("@lune/fs") :: any
local roblox = require("@lune/roblox") :: any

local placeDirs = {}
for _, dirName in fs.readDir("tests/roblox/rbx-test-files/places") do
	table.insert(placeDirs, "tests/roblox/rbx-test-files/places/" .. dirName)
end

for _, placeDir in placeDirs do
	local placeBinary = roblox.readPlaceFile(placeDir .. "/binary.rbxl")
	local placeXml = roblox.readPlaceFile(placeDir .. "/xml.rbxlx")

	assert(placeBinary.ClassName == "DataModel")
	assert(placeXml.ClassName == "DataModel")

	assert(placeBinary:IsA("ServiceProvider"))
	assert(placeXml:IsA("ServiceProvider"))
end
