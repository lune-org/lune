local net = require("@lune/net")

local QUERY: { [string]: string } = {
	Key = "Value",
	Hello = "World",
	SpaceEmoji = " 🚀 ",
}

-- Make a request with some basic query params as well
-- as a special non-ascii one that needs url encoding

local response = net.request({
	url = "https://httpbingo.org/anything",
	query = QUERY,
})

assert(
	response.ok,
	"Request failed with status "
		.. tostring(response.statusCode)
		.. " "
		.. tostring(response.statusMessage)
)

-- We should get a json response here with an "args" table which is our query

local success, json = pcall(net.jsonDecode, response.body)
assert(success, "Failed to decode json response\n" .. tostring(json))

local args = if type(json.args) == "table" then json.args else nil
assert(args ~= nil, "Response body did not contain an args table")

-- The args table should then have the same contents as our query,
-- these will however be returned as arrays of strings and not a
-- single string, presumably because http query params support
-- multiple values of the same name, so we just grab the first

for key, value in QUERY do
	local received = args[key][1]
	if received == nil then
		error(string.format("Response body did not contain query parameter '%s'", key))
	elseif typeof(received) ~= typeof(value) then
		error(
			string.format(
				"Response body contained query parameter '%s' but it was of type '%s', expected '%s'",
				key,
				typeof(received),
				typeof(value)
			)
		)
	elseif received ~= value then
		error(
			string.format(
				"Response body contained query parameter '%s' but it had the value '%s', expected '%s'",
				key,
				received,
				value
			)
		)
	end
end
