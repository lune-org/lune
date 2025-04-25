local net = require("@lune/net")
local roblox = require("@lune/roblox")
local serde = require("@lune/serde")

roblox.implementMethod("HttpService", "GetAsync", function(_, url: string)
	local response = net.request({
		method = "GET",
		url = url,
	})
	return response.body
end)

roblox.implementMethod("HttpService", "JSONDecode", function(_, value)
	return serde.decode("json", value)
end)

-- Reference: https://create.roblox.com/docs/reference/engine/classes/HttpService#GetAsync

local URL_ASTROS = "http://api.open-notify.org/astros.json"

local game = roblox.Instance.new("DataModel")
local HttpService = game:GetService("HttpService") :: any

local response = HttpService:GetAsync(URL_ASTROS)
local data = HttpService:JSONDecode(response)

assert(type(data) == "table", "Returned JSON data should decode to a table")
assert(data.message == "success", "Returned JSON data should have a 'message' with value 'success'")
assert(type(data.people) == "table", "Returned JSON data should have a 'people' table")
