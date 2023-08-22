local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

-- Spawn an asynchronous background task (eg. web server)

local PORT = 8082

task.delay(3, function()
	stdio.ewrite("Test did not complete in time\n")
	task.wait(1)
	process.exit(1)
end)

local handle = net.serve(PORT, function(request)
	return ""
end)

-- Require modules same way we did in the async_concurrent and async_sequential tests

local module3
local module4

task.defer(function()
	module4 = require("./modules/async")
end)

task.spawn(function()
	module3 = require("./modules/async")
end)

local _module1 = require("./modules/async")
local _module2 = require("./modules/async")

task.wait(1)

assert(type(module3) == "table", "Required module3 did not return a table")
assert(module3.Foo == "Bar", "Required module3 did not contain correct values")
assert(module3.Hello == "World", "Required module3 did not contain correct values")

assert(type(module4) == "table", "Required module4 did not return a table")
assert(module4.Foo == "Bar", "Required module4 did not contain correct values")
assert(module4.Hello == "World", "Required module4 did not contain correct values")

assert(module3 == module4, "Required modules should point to the same return value")

-- Stop the server and exit successfully

handle.stop()
process.exit(0)
