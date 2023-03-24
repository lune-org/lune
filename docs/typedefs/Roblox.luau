export type Instance = {}

--[=[
	@class Process

	Built-in library for manipulating Roblox place & model files

	### Example usage

	```lua
	local roblox = require("@lune/roblox")

	-- Reading & writing a place file
	local game = roblox.readPlaceFile("myPlaceFile.rbxl")
	local workspace = game:GetService("Workspace")

	for _, child in workspace:GetChildren() do
		print("Found child " .. child.Name .. " of class " .. child.ClassName)
	end

	roblox.writePlaceFile("myPlaceFile.rbxl", game)
	```
]=]
export type Roblox = {
	--[=[
		Reads a place file into a DataModel instance.

		### Example usage

		```lua
		local roblox = require("@lune/roblox")
		local game = roblox.readPlaceFile("filePath.rbxl")
		```
	]=]
	readPlaceFile: (filePath: string) -> Instance,
	--[=[
		Reads a model file into a table of instances.

		### Example usage

		```lua
		local roblox = require("@lune/roblox")
		local instances = roblox.readModelFile("filePath.rbxm")
		```
	]=]
	readModelFile: (filePath: string) -> { Instance },
	--[=[
		Writes a DataModel instance to a place file.

		### Example usage

		```lua
		local roblox = require("@lune/roblox")
		roblox.writePlaceFile("filePath.rbxl", game)
		```
	]=]
	writePlaceFile: (filePath: string, dataModel: Instance) -> (),
	--[=[
		Writes one or more instances to a model file.

		### Example usage

		```lua
		local roblox = require("@lune/roblox")
		roblox.writeModelFile("filePath.rbxm", { instance1, instance2, ... })
		```
	]=]
	writeModelFile: (filePath: string, instances: { Instance }) -> (),
	--[=[
		Gets the current auth cookie, for usage with Roblox web APIs.

		### Example usage

		```lua
		local roblox = require("@lune/roblox")
		local net = require("@lune/net")

		local cookie = roblox.getAuthCookie()
		assert(cookie ~= nil, "Failed to get roblox auth cookie")

		local myPrivatePlaceId = 1234567890

		local response = net.request({
			url = "https://assetdelivery.roblox.com/v2/assetId/" .. tostring(myPrivatePlaceId),
			headers = {
				Cookie = cookie,
			},
		})

		local responseTable = net.jsonDecode(response.body)
		local responseLocation = responseTable.locations[1].location
		print("Download link to place: " .. responseLocation)
		```
	]=]
	getAuthCookie: (raw: boolean?) -> string?,
}
