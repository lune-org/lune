local net = require("@lune/net")

-- NOTE: We do not use httpbingo in these TCP tests, since they do not accept
-- very basic HTTP requests such as the below, and it is not known which headers
-- are missing for them to work. All we need is a bit of HTML and a server that
-- properly closes the connection, which example.com handles just fine.
local stream = net.tcp.connect("example.com", 80)

local request = "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n"
stream:write(request)

local responseChunks = {}
while true do
	local chunk = stream:read()
	if chunk == nil or #chunk <= 0 then
		break
	end
	table.insert(responseChunks, chunk)
end

local response = table.concat(responseChunks)
local function contains(substring: string): boolean
	return string.find(response, substring, 1, true) ~= nil
end

stream:close()

assert(contains("HTTP/1.1 200"), "Should receive HTTP 200 response")
assert(contains("\nContent-Length: "), "Response should have content length header")
assert(contains("<html"), "Response should contain some HTML")
assert(contains("/html>"), "Response should contain some HTML")
