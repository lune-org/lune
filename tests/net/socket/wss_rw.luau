local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

-- We're going to use Discord's WebSocket gateway server
-- for testing that we can both read from a stream,
-- as well as write to the same stream concurrently
local socket = net.socket("wss://gateway.discord.gg/?v=10&encoding=json")

local spawnedThread = task.spawn(function()
	while not socket.closeCode do
		local message = socket:next()
		if message ~= nil then
			assert(type(message) == "string", "Message should be a string")
			assert(#message > 0, "Message should not be empty")
			assert(string.sub(message, 1, 1) == "{", "Message should be a JSON object")
		end
	end
end)

local delayedThread = task.delay(10, function()
	stdio.ewrite("`socket.send` halted, failed to write to socket")
	process.exit(1)
end)

local payload = '{"op":1,"d":null}'
socket:send(payload)
socket:send(buffer.fromstring(payload))
socket:close(1000)

task.cancel(delayedThread)
task.cancel(spawnedThread)

process.exit(0)
