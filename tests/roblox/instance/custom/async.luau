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

local game = roblox.Instance.new("DataModel")
local HttpService = game:GetService("HttpService") :: any

local response = HttpService:GetAsync("https://httpbingo.org/json")
local data = HttpService:JSONDecode(response)

assert(type(data) == "table", "Returned JSON data should decode to a table")
assert(type(data.slideshow) == "table", "Returned JSON data should contain 'slideshow'")
