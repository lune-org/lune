local net = require("@lune/net")
local serde = require("@lune/serde")

-- Should decompress automatically by default

local response = net.request({
	url = "https://httpbingo.org/gzip",
	headers = {
		["Accept-Encoding"] = "gzip",
	} :: { [string]: string },
})

assert(
	response.ok,
	"Request failed with status "
		.. tostring(response.statusCode)
		.. " "
		.. tostring(response.statusMessage)
)

local success, json = pcall(serde.decode, "json" :: "json", response.body)
assert(success, "Failed to decode json response\n" .. tostring(json))

-- Content encoding header should no longer exist when automatically decompressed

assert(
	response.headers["content-encoding"] == nil,
	"Content encoding header still exists after automatic decompression"
)

-- Should do nothing when explicitly disabled

local response2 = net.request({
	url = "https://httpbingo.org/gzip",
	headers = {
		["Accept-Encoding"] = "gzip",
	} :: { [string]: string },
	options = { decompress = false },
})

assert(
	response2.ok,
	"Request failed with status "
		.. tostring(response2.statusCode)
		.. " "
		.. tostring(response2.statusMessage)
)

local success2 = pcall(serde.decode, "json" :: "json", response2.body)
assert(not success2, "Decompression disabled still returned json response")

-- Content encoding header should still exist when not automatically decompressed

assert(
	response2.headers["content-encoding"] ~= nil,
	"Content encoding header is missing when automatic decompression is disabled"
)
