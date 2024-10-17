--> A basic web socket server that echoes given messages

local net = require("@lune/net")
local process = require("@lune/process")
local task = require("@lune/task")

local PORT = if process.env.PORT ~= nil and #process.env.PORT > 0
	then assert(tonumber(process.env.PORT), "Failed to parse port from env")
	else 8080

-- Run the server on port 8080, if we get a normal http request on
-- the port this will respond with 426 Upgrade Required by default

local handle = net.serve(PORT, {
	handleWebSocket = function(socket)
		print("Got new web socket connection!")
		repeat
			local message = socket:next()
			if message ~= nil then
				socket:send("Echo - " .. message)
			end
		until message == nil
		print("Web socket disconnected.")
	end,
})

print(`Listening on port {PORT} 🚀`)

-- Exit our example after a small delay, if you copy this
-- example just remove this part to keep the server running

task.delay(10, function()
	print("Shutting down...")
	task.wait(1)
	handle.stop()
	task.wait(1)
end)
