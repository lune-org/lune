local net = require("@lune/net")

local stream = net.tcp.connect("httpbingo.org", 80)

local request = "GET /get HTTP/1.1\r\nHost: httpbingo.org\r\nConnection: close\r\n\r\n"
stream:write(request)

local responseChunks = {}
while true do
	local chunk = stream:read()
	if not chunk then
		break
	end
	table.insert(responseChunks, chunk)
end
local response = table.concat(responseChunks)

stream:close()

assert(string.find(response, "HTTP/1.1 200"), "Should receive HTTP 200 response")
assert(string.find(response, "httpbingo.org"), "Response should contain the host")
assert(
	string.find(response, "Content-Length") or string.find(response, "Transfer-Encoding"),
	"Response should have content length or transfer encoding"
)
