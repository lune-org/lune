local net = require("@lune/net")

local stream = net.tcp.connect("httpbingo.org", 80)

assert(type(stream.localIp) == "string", "localIp should be a string")
assert(type(stream.localPort) == "number", "localPort should be a number")
assert(type(stream.remoteIp) == "string", "remoteIp should be a string")
assert(stream.remotePort == 80, "remotePort should be 80")

assert(
	string.match(stream.remoteIp, "^%d+%.%d+%.%d+%.%d+$"),
	"remoteIp should be a valid IP address"
)

assert(stream.localPort > 0 and stream.localPort <= 65535, "localPort should be in valid range")

stream:close()
