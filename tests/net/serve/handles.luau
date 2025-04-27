local net = require("@lune/net")
local task = require("@lune/task")

local PORT = 8083
local URL = `http://127.0.0.1:{PORT}`
local RESPONSE = "Hello, lune!"

local handle = net.serve(PORT, function(request)
	return RESPONSE
end)

-- Stopping is not guaranteed to happen instantly since it is async, but
-- it should happen on the next yield, so we wait the minimum amount here

handle.stop()
task.wait()

-- Sending a request to the stopped server should now error

local success, response2 = pcall(net.request, URL)
if not success then
	local message = tostring(response2)
	assert(
		string.find(message, "Connection reset")
			or string.find(message, "Connection closed")
			or string.find(message, "Connection refused")
			or string.find(message, "No connection could be made"), -- Windows Request Error
		"Server did not stop responding to requests"
	)
else
	assert(not response2.ok, "Server did not stop responding to requests")
end

--[[
	Trying to *stop* the server again should error, and
	also mention that the server has already been stopped

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
