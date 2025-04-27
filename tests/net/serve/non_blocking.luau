local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local PORT = 8082
local RESPONSE = "Hello, lune!"

-- Serve should not yield the entire main thread forever, only
-- for the initial binding to socket which should be very fast

local thread = task.delay(1, function()
	stdio.ewrite("Serve must not yield the current thread for too long\n")
	task.wait(1)
	process.exit(1)
end)

local handle = net.serve(PORT, function(request)
	return RESPONSE
end)

task.cancel(thread)

handle:stop()
