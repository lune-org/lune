local PORT = 8080
local URL = `http://127.0.0.1:{PORT}`
local WS_URL = `ws://127.0.0.1:{PORT}`
local REQUEST = "Hello from client!"
local RESPONSE = "Hello, lune!"

local thread = task.delay(0.2, function()
	task.spawn(error, "Serve must not block the current thread")
	process.exit(1)
end)

--[[
	Serve should also take a full config with handler functions

	A server should also be able to start on a previously closed port
]]

local handle = net.serve(PORT, function(request)
	return RESPONSE
end)

task.cancel(thread)
handle.stop()
task.wait()

local handle2 = net.serve(PORT, {
	handleRequest = function()
		return RESPONSE
	end,
	handleWebSocket = function(socket)
		local socketMessage = socket.next()
		assert(socketMessage == REQUEST, "Invalid web socket request from client")
		socket.send(RESPONSE)
		socket.close()
	end,
})

local response = net.request(URL).body
assert(response == RESPONSE, "Invalid response from server")

-- Web socket client should work
local socket = net.socket(WS_URL)

socket.send(REQUEST)

local socketMessage = socket.next()
assert(socketMessage ~= nil, "Got no web socket response from server")
assert(socketMessage == RESPONSE, "Invalid web socket response from server")

socket.close()

-- Wait for the socket to close and make sure we can't send messages afterwards
task.wait()
local success3, err2 = (pcall :: any)(socket.send, "")
assert(not success3, "Sending messages after the socket has been closed should error")
local message2 = tostring(err2)
assert(
	string.find(message2, "close") or string.find(message2, "closing"),
	"The error message for sending messages on a closed web socket should be descriptive"
)

-- Stop the server to end the test
handle2.stop()
