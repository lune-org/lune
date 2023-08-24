local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local PORT = 8081
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
		stdio.ewrite("Web socket should upgrade automatically, not pass to the request handler\n")
		task.wait(1)
		process.exit(1)
		return "unreachable"
	end,
	handleWebSocket = function(socket)
		local socketMessage = socket.next()
		assert(socketMessage == REQUEST, "Invalid web socket request from client")
		socket.send(RESPONSE)
		socket.close()
	end,
})

task.cancel(thread)

-- Web socket responses should also be responded to

local thread2 = task.delay(1, function()
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

task.cancel(thread2)

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
