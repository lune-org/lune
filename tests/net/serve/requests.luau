local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local PORT = 8084
local URL = `http://127.0.0.1:{PORT}`
local RESPONSE = "Hello, lune!"

-- Serve should get proper path, query, and other request information

local handle = net.serve(PORT, function(request)
	-- print("Got a request from", request.ip, "on port", request.port)

	assert(type(request.path) == "string")
	assert(type(request.query) == "table")
	assert(type(request.query.key) == "table")
	assert(type(request.query.key2) == "string")

	assert(request.path == "/some/path")
	assert(request.query.key[1] == "param1")
	assert(request.query.key[2] == "param2")
	assert(request.query.key2 == "param3")

	return RESPONSE
end)

-- Serve should be able to handle at least 1000 requests per second with a basic handler such as the above

local thread = task.delay(1, function()
	stdio.ewrite("Serve should respond to requests in a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

-- Serve should respond to requests we send, and keep responding until we stop it

for _ = 1, 1024 do
	local response = net.request(URL .. "/some/path?key=param1&key=param2&key2=param3").body
	assert(response == RESPONSE, "Invalid response from server")
end

task.cancel(thread)

handle.stop()
