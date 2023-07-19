local net = require("@lune/net")
local process = require("@lune/process")
local task = require("@lune/task")

-- We're going to use Discord's WebSocket gateway server
-- for testing that we can both read from a stream,
-- as well as write to the same stream concurrently
local socket = net.socket("wss://gateway.discord.gg/?v=10&encoding=json")

local spawnedThread = task.spawn(function()
	while not socket.closeCode do
		socket.next()
	end
end)

local delayedThread = task.delay(5, function()
	task.defer(process.exit, 1)
	error("`socket.send` halted, failed to write to socket")

	process.exit(1)
end)

task.wait(1)

socket.send('{"op":1,"d":null}')
socket.close(1000)

task.cancel(delayedThread)
task.cancel(spawnedThread)
