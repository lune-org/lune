local net = require("@lune/net")

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
assert(contains("<html>"), "Response should contain some HTML")
