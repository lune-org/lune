--> A basic web socket client that communicates with an echo server

local PORT = if process.env.PORT ~= nil and #process.env.PORT > 0
	then assert(tonumber(process.env.PORT), "Failed to parse port from env")
	else 8080

local URL = `ws://127.0.0.1:{PORT}`

-- Connect to our web socket server

local socket = net.socket(URL)

print("Connected to echo web socket server at '" .. URL .. "'")
print("Sending a message every second for 5 seconds...")

-- Force exit after 10 seconds in case the server is not responding well

local forceExit = task.delay(10, function()
	warn("Example did not complete in time, exiting...")
	process.exit(1)
end)

-- Send one message per second and time it

for i = 1, 5 do
	local start = os.clock()
	socket.send(tostring(1))
	local response = socket.next()
	local elapsed = os.clock() - start
	print(`Got response '{response}' in {elapsed * 1_000} milliseconds`)
	task.wait(1 - elapsed)
end

-- Everything went well, and we are done with the socket, so we can close it

print("Closing web socket...")
socket.close()

task.cancel(forceExit)
print("Done! 🌙")
