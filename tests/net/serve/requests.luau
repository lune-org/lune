local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local PORT = 8083
local URL = `http://127.0.0.1:{PORT}`
local RESPONSE = "Hello, lune!"

-- Serve should respond to a request we send to it

local handle = net.serve(PORT, function(request)
	assert(request.path == "/some/path")
	assert(request.query.key == "param2")
	assert(request.query.key2 == "param3")
	return RESPONSE
end)

local thread = task.delay(1, function()
	stdio.ewrite("Serve should respond to requests in a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

local response = net.request(URL .. "/some/path?key=param1&key=param2&key2=param3").body
assert(response == RESPONSE, "Invalid response from server")

task.cancel(thread)

handle.stop()
