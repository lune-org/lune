local net = require("@lune/net")
local process = require("@lune/process")
local stdio = require("@lune/stdio")
local task = require("@lune/task")

local PORT = 8866
local URL = `http://127.0.0.1:{PORT}`
local RESPONSE = "Hello, lune!"

-- Serve should accept all of the documented response shapes

local handle = net.serve(PORT, function(request)
	if request.path == "/string" then
		return RESPONSE
	elseif request.path == "/empty" then
		return {}
	elseif request.path == "/body" then
		return { body = RESPONSE }
	elseif request.path == "/status" then
		return { status = 418 }
	elseif request.path == "/headers" then
		return { headers = { ["x-lune"] = "yes" } }
	end
	return { status = 404 }
end)

-- Serve should respond to all of the requests below in a reasonable amount of time

local thread = task.delay(1, function()
	stdio.ewrite("Serve should respond to requests in a reasonable amount of time\n")
	task.wait(1)
	process.exit(1)
end)

-- Strings should be sent as-is with a status of 200

local response = net.request(URL .. "/string")
assert(response.statusCode == 200, "String response should have status 200")
assert(response.body == RESPONSE, "String response should have the given body")

-- Tables should default to a status of 200 and an empty body

local response2 = net.request(URL .. "/empty")
assert(response2.statusCode == 200, "Empty table response should have status 200")
assert(response2.body == "", "Empty table response should have an empty body")

local response3 = net.request(URL .. "/body")
assert(response3.statusCode == 200, "Table response without status should have status 200")
assert(response3.body == RESPONSE, "Table response without status should have the given body")

-- Tables should use the given status and headers when present

local response4 = net.request(URL .. "/status")
assert(response4.statusCode == 418, "Table response with status should have the given status")

local response5 = net.request(URL .. "/headers")
assert(response5.statusCode == 200, "Table response with headers should have status 200")
assert(
	response5.headers["x-lune"] == "yes",
	"Table response with headers should have the given headers"
)

task.cancel(thread)

handle:stop()
