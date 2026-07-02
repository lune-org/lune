local net = require("@lune/net")
local task = require("@lune/task")

-- We're going to use Discord's WebSocket gateway server
-- for testing wss - it does not require auth, and this test
-- only exists to ensure wss (WebSockets with TLS) works correctly
local socket = net.socket("wss://gateway.discord.gg/?v=10&encoding=json")

while not socket.closeCode do
	local message = socket:next()

	if message ~= nil then
		-- Make sure we got some JSON object, which Discord should always give us
		assert(type(message) == "string", "Message should be a string")
		assert(#message > 0, "Message should not be empty")
		assert(string.sub(message, 1, 1) == "{", "Message should be a JSON object")

		-- Close the connection after a second with the success close code
		task.wait(1)
		socket:close(1000)
	end
end
