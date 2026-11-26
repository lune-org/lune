local net = require("@lune/net")

local PORT = 8853
local URL = `http://127.0.0.1:{PORT}`

local handle = net.serve(PORT, function(request)
	if request.path == "/redirect" then
		return {
			status = 302,
			headers = { Location = "/echo" },
		}
	end
	return request.headers.host or "<missing>"
end)

-- The Host header must carry a non-default port (RFC 9110 section 7.2)
local response = net.request(URL .. "/echo")
assert(response.body == `127.0.0.1:{PORT}`, `Expected Host header with port, got '{response.body}'`)

-- A Host header set by the caller must be kept
response = net.request({
	url = URL .. "/echo",
	headers = { Host = "example.com" },
})
assert(response.body == "example.com", `Expected custom Host header, got '{response.body}'`)

-- A relative redirect must still send the Host header
response = net.request(URL .. "/redirect")
assert(response.body == `127.0.0.1:{PORT}`, `Expected Host header after redirect, got '{response.body}'`)

handle:stop()
