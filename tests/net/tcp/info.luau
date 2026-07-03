local net = require("@lune/net")

local stream = net.tcp.connect("httpbingo.org", 80)

assert(type(stream.localIp) == "string", "localIp should be a string")
assert(type(stream.localPort) == "number", "localPort should be a number")
assert(type(stream.remoteIp) == "string", "remoteIp should be a string")
assert(stream.remotePort == 80, "remotePort should be 80")

-- connection may be made over IPv4 or IPv6
local function isIpAddress(ip: string): boolean
	if string.match(ip, "^%d+%.%d+%.%d+%.%d+$") then
		return true -- IPv4
	end
	-- IPv6
	return string.find(ip, ":") ~= nil and string.match(ip, "^[%x:%.]+$") ~= nil
end

assert(isIpAddress(stream.remoteIp), "remoteIp should be a valid IP address")

assert(stream.localPort > 0 and stream.localPort <= 65535, "localPort should be in valid range")

stream:close()
