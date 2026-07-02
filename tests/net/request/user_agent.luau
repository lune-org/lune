local net = require("@lune/net")
local serde = require("@lune/serde")

local runtime, version = table.unpack(_VERSION:split(" "))
local expectedUserAgent = runtime:lower() .. "/" .. version

local userAgent: string =
	serde.decode("json", net.request("https://www.whatsmyua.info/api/v1/ua").body)[1].ua.rawUa

assert(userAgent == expectedUserAgent, "Expected user agent to be " .. expectedUserAgent)
