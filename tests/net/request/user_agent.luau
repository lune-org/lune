local net = require("@lune/net")

local runtime, version = table.unpack(_VERSION:split(" "))
local expectedUserAgent = runtime:lower() .. "/" .. version

local userAgent: string =
	net.jsonDecode(net.request("https://www.whatsmyua.info/api/v1/ua").body)[1].ua.rawUa

assert(userAgent == expectedUserAgent, "Expected user agent to be " .. expectedUserAgent)
