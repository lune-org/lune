local PORT = 8080
local URL = `http://127.0.0.1:{PORT}`
local WS_URL = `ws://127.0.0.1:{PORT}`
local REQUEST = "Hello from client!"
local RESPONSE = "Hello, lune!"

local handle = net.serve(PORT, function(request)
	-- info("Request:", request)
	-- info("Responding with", RESPONSE)
	assert(request.path == "/some/path")
	assert(request.query.key == "param2")
	assert(request.query.key2 == "param3")
	return RESPONSE
end)

local response = net.request(URL .. "/some/path?key=param1&key=param2&key2=param3").body
assert(response == RESPONSE, "Invalid response from server")

handle.stop()

-- Stopping is not guaranteed to happen instantly since it is async, but
-- it should happen on the next yield, so we wait the minimum amount here
task.wait()

-- Sending a net request may error if there was
-- a connection issue, we should handle that here
local success, response2 = pcall(net.request, URL)
if not success then
	local message = tostring(response2)
	assert(
		string.find(message, "Connection reset")
			or string.find(message, "Connection closed")
			or string.find(message, "Connection refused"),
		"Server did not stop responding to requests"
	)
else
	assert(not response2.ok, "Server did not stop responding to requests")
end

--[[
	Trying to stop the server again should error and
	mention that the server has already been stopped

	Note that we cast pcall to any because of a
	Luau limitation where it throws a type error for
	`err` because handle.stop doesn't return any value
]]
local success2, err = (pcall :: any)(handle.stop)
assert(not success2, "Calling stop twice on the net serve handle should error")
local message = tostring(err)
assert(
	string.find(message, "stop")
		or string.find(message, "shutdown")
		or string.find(message, "shut down"),
	"The error message for calling stop twice on the net serve handle should be descriptive"
)

--[[
	Serve should also take a full config with handler functions

	A server should also be able to start on the previously closed port
]]
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

local response3 = net.request(URL).body
assert(response3 == RESPONSE, "Invalid response from server")

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
