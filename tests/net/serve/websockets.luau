local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local PORT = 8081
local URL = `http://127.0.0.1:{PORT}`
local WS_URL = `ws://127.0.0.1:{PORT}`
local REQUEST = "Hello from client!"
local RESPONSE = "Hello, lune!"

-- Serve should not block the thread from continuing

local thread = task.delay(1, function()
	stdio.ewrite("Serve must not block the current thread\n")
	task.wait(1)
	process.exit(1)
end)

local handle = net.serve(PORT, {
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

task.cancel(thread)

-- Serve should respond to a request we send to it

local thread2 = task.delay(1, function()
	stdio.ewrite("Serve should respond to requests in a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local response = net.request(URL).body
assert(response == RESPONSE, "Invalid response from server")

task.cancel(thread2)

-- Web socket responses should also be responded to

local thread3 = task.delay(1, function()
	stdio.ewrite("Serve should respond to websockets in a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local socket = net.socket(WS_URL)

socket.send(REQUEST)

local socketMessage = socket.next()
assert(socketMessage ~= nil, "Got no web socket response from server")
assert(socketMessage == RESPONSE, "Invalid web socket response from server")

socket.close()

task.cancel(thread3)

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
handle.stop()
