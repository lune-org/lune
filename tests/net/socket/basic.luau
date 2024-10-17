local net = require("@lune/net")

-- We're going to use Discord's WebSocket gateway server for testing
local socket = net.socket("wss://gateway.discord.gg/?v=10&encoding=json")

assert(type(socket.next) == "function", "next must be a function")
assert(type(socket.send) == "function", "send must be a function")
assert(type(socket.close) == "function", "close must be a function")

-- Request to close the socket
socket:close()

-- Drain remaining messages, until we got our close message
while socket:next() do
end

assert(type(socket.closeCode) == "number", "closeCode should exist after closing")
assert(socket.closeCode == 1000, "closeCode should be 1000 after closing")

local success, message = pcall(function()
	socket:send("Hello, world!")
end)

assert(not success, "send should fail after closing")
assert(
	string.find(tostring(message), "closed") or string.find(tostring(message), "closing"),
	"send should fail with a message that the socket was closed"
)
