--!strict

local fs = require("@lune/fs")
local net = require("@lune/net")
local serde = require("@lune/serde")
local process = require("@lune/process")
local roblox = require("@lune/roblox")

export type LuneDataModel = roblox.DataModel
export type LuneInstance = roblox.Instance

local function getAuthCookieWithFallbacks()
	local cookie = roblox.getAuthCookie()
	if cookie then
		return cookie
	end

	local cookieFromEnv = process.env.REMODEL_AUTH
	if cookieFromEnv and #cookieFromEnv > 0 then
		return `.ROBLOSECURITY={cookieFromEnv}`
	end

	for index, arg in process.args do
		if arg == "--auth" then
			local cookieFromArgs = process.args[index + 1]
			if cookieFromArgs and #cookieFromArgs > 0 then
				return `.ROBLOSECURITY={cookieFromArgs}`
			end
			break
		end
	end

	error([[
		Failed to find ROBLOSECURITY cookie for authentication!
		Make sure you have logged into studio, or set the ROBLOSECURITY environment variable.
	]])
end

local function downloadAssetId(assetId: number)
	-- 1. Try to find the auth cookie for the current user
	local cookie = getAuthCookieWithFallbacks()

	-- 2. Send a request to the asset delivery API,
	--    which will respond with cdn download link(s)
	local assetApiResponse = net.request({
		url = `https://assetdelivery.roblox.com/v2/assetId/{assetId}`,
		headers = {
			Accept = "application/json",
			Cookie = cookie,
		},
	})
	if not assetApiResponse.ok then
		error(
			string.format(
				"Failed to fetch asset download link for asset id %s!\n%s (%s)\n%s",
				tostring(assetId),
				tostring(assetApiResponse.statusCode),
				tostring(assetApiResponse.statusMessage),
				tostring(assetApiResponse.body)
			)
		)
	end

	-- 3. Make sure we got a valid response body
	local assetApiBody = serde.decode("json", assetApiResponse.body)
	if type(assetApiBody) ~= "table" then
		error(
			string.format(
				"Asset delivery API returned an invalid response body!\n%s",
				assetApiResponse.body
			)
		)
	elseif type(assetApiBody.locations) ~= "table" then
		error(
			string.format(
				"Asset delivery API returned an invalid response body!\n%s",
				assetApiResponse.body
			)
		)
	end

	-- 4. Grab the first asset download location - we only
	--    requested one in our query, so this will be correct
	local firstLocation = assetApiBody.locations[1]
	if type(firstLocation) ~= "table" then
		error(
			string.format(
				"Asset delivery API returned no download locations!\n%s",
				assetApiResponse.body
			)
		)
	elseif type(firstLocation.location) ~= "string" then
		error(
			string.format(
				"Asset delivery API returned no valid download locations!\n%s",
				assetApiResponse.body
			)
		)
	end

	-- 5. Fetch the place contents from the cdn
	local cdnResponse = net.request({
		url = firstLocation.location,
		headers = {
			Cookie = cookie,
		},
	})
	if not cdnResponse.ok then
		error(
			string.format(
				"Failed to download asset with id %s from the Roblox cdn!\n%s (%s)\n%s",
				tostring(assetId),
				tostring(cdnResponse.statusCode),
				tostring(cdnResponse.statusMessage),
				tostring(cdnResponse.body)
			)
		)
	end

	-- 6. The response body should now be the contents of the asset file
	return cdnResponse.body
end

local function uploadAssetId(assetId: number, contents: string)
	-- 1. Try to find the auth cookie for the current user
	local cookie = getAuthCookieWithFallbacks()

	-- 2. Create request headers in advance, we might re-use them for CSRF challenges
	local headers = {
		["User-Agent"] = "Roblox/WinInet",
		["Content-Type"] = "application/octet-stream",
		Accept = "application/json",
		Cookie = cookie,
	}

	-- 3. Create and send a request to the upload url
	local uploadResponse = net.request({
		url = `https://data.roblox.com/Data/Upload.ashx?assetid={assetId}`,
		body = contents,
		method = "POST",
		headers = headers,
	})

	-- 4. Check if we got a valid response, we might have gotten a CSRF
	--    challenge and need to send the request with a token included
	if
		not uploadResponse.ok
		and uploadResponse.statusCode == 403
		and uploadResponse.headers["x-csrf-token"] ~= nil
	then
		headers["X-CSRF-Token"] = uploadResponse.headers["x-csrf-token"]
		uploadResponse = net.request({
			url = `https://data.roblox.com/Data/Upload.ashx?assetid={assetId}`,
			body = contents,
			method = "POST",
			headers = headers,
		})
	end

	if not uploadResponse.ok then
		error(
			string.format(
				"Failed to upload asset with id %s to Roblox!\n%s (%s)\n%s",
				tostring(assetId),
				tostring(uploadResponse.statusCode),
				tostring(uploadResponse.statusMessage),
				tostring(uploadResponse.body)
			)
		)
	end
end

local remodel = {}

--[=[
	Load an `rbxl` or `rbxlx` file from the filesystem.

	Returns a `DataModel` instance, equivalent to `game` from within Roblox.
]=]
function remodel.readPlaceFile(filePath: string)
	local placeFile = fs.readFile(filePath)
	local place = roblox.deserializePlace(placeFile)
	return place
end

--[=[
	Load an `rbxm` or `rbxmx` file from the filesystem.

	Note that this function returns a **list of instances** instead of a single instance!
	This is because models can contain mutliple top-level instances.
]=]
function remodel.readModelFile(filePath: string)
	local modelFile = fs.readFile(filePath)
	local model = roblox.deserializeModel(modelFile)
	return model
end

--[=[
	Reads a place asset from Roblox, equivalent to `remodel.readPlaceFile`.

	***NOTE:** This function requires authentication using a ROBLOSECURITY cookie!*
]=]
function remodel.readPlaceAsset(assetId: number)
	local contents = downloadAssetId(assetId)
	local place = roblox.deserializePlace(contents)
	return place
end

--[=[
	Reads a model asset from Roblox, equivalent to `remodel.readModelFile`.

	***NOTE:** This function requires authentication using a ROBLOSECURITY cookie!*
]=]
function remodel.readModelAsset(assetId: number)
	local contents = downloadAssetId(assetId)
	local place = roblox.deserializeModel(contents)
	return place
end

--[=[
	Saves an `rbxl` or `rbxlx` file out of the given `DataModel` instance.

	If the instance is not a `DataModel`, this function will throw.
	Models should be saved with `writeModelFile` instead.
]=]
function remodel.writePlaceFile(filePath: string, dataModel: LuneDataModel)
	local asBinary = string.sub(filePath, -5) == ".rbxl"
	local asXml = string.sub(filePath, -6) == ".rbxlx"
	assert(asBinary or asXml, "File path must have .rbxl or .rbxlx extension")
	local placeFile = roblox.serializePlace(dataModel, asXml)
	fs.writeFile(filePath, placeFile)
end

--[=[
	Saves an `rbxm` or `rbxmx` file out of the given `Instance`.

	If the instance is a `DataModel`, this function will throw.
	Places should be saved with `writePlaceFile` instead.
]=]
function remodel.writeModelFile(filePath: string, instance: LuneInstance)
	local asBinary = string.sub(filePath, -5) == ".rbxm"
	local asXml = string.sub(filePath, -6) == ".rbxmx"
	assert(asBinary or asXml, "File path must have .rbxm or .rbxmx extension")
	local placeFile = roblox.serializeModel({ instance }, asXml)
	fs.writeFile(filePath, placeFile)
end

--[=[
	Uploads the given `DataModel` instance to Roblox, overwriting an existing place.

	If the instance is not a `DataModel`, this function will throw.
	Models should be uploaded with `writeExistingModelAsset` instead.

	***NOTE:** This function requires authentication using a ROBLOSECURITY cookie!*
]=]
function remodel.writeExistingPlaceAsset(dataModel: LuneDataModel, assetId: number)
	local placeFile = roblox.serializePlace(dataModel)
	uploadAssetId(assetId, placeFile)
end

--[=[
	Uploads the given instance to Roblox, overwriting an existing model.

	If the instance is a `DataModel`, this function will throw.
	Places should be uploaded with `writeExistingPlaceAsset` instead.

	***NOTE:** This function requires authentication using a ROBLOSECURITY cookie!*
]=]
function remodel.writeExistingModelAsset(instance: LuneInstance, assetId: number)
	local modelFile = roblox.serializeModel({ instance })
	uploadAssetId(assetId, modelFile)
end

remodel.readFile = fs.readFile
remodel.readDir = fs.readDir
remodel.writeFile = fs.writeFile
remodel.createDirAll = fs.writeDir
remodel.removeFile = fs.removeFile
remodel.removeDir = fs.removeDir
remodel.isFile = fs.isFile
remodel.isDir = fs.isDir

return remodel
