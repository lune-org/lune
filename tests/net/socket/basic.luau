local net = require("@lune/net")

-- We're going to use Discord's WebSocket gateway server for testing
local socket = net.socket("wss://gateway.discord.gg/?v=10&encoding=json")

assert(type(socket.next) == "function", "next must be a function")
assert(type(socket.send) == "function", "send must be a function")
assert(type(socket.close) == "function", "close must be a function")

-- Request to close the socket
socket:close()

-- Drain remaining messages, until we got our close message
repeat
	local message = socket:next()
	if message ~= nil then
		assert(type(message) == "string", "Message should be a string")
		assert(#message > 0, "Message should not be empty")
		assert(string.sub(message, 1, 1) == "{", "Message should be a JSON object")
	end
until message == nil

assert(type(socket.closeCode) == "number", "closeCode should exist after closing")
assert(socket.closeCode == 1000, "closeCode should be 1000 after closing")

local success, errorMessage = pcall(function()
	socket:send("Hello, world!")
end)

assert(not success, "send should fail after closing")
assert(
	string.find(tostring(errorMessage), "closed") or string.find(tostring(errorMessage), "closing"),
	"send should fail with a message that the socket was closed"
)
