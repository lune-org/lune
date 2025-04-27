--> A basic http server that echoes the given request
--> body at /ping and otherwise responds 404 "Not Found"

local net = require("@lune/net")
local process = require("@lune/process")
local task = require("@lune/task")

local PORT = if process.env.PORT ~= nil and #process.env.PORT > 0
	then assert(tonumber(process.env.PORT), "Failed to parse port from env")
	else 8080

-- Create our responder functions

local function pong(request: net.ServeRequest): string
	return `Pong!\n{request.path}\n{request.body}`
end

local function teapot(_request: net.ServeRequest): net.ServeResponse
	return {
		status = 418,
		body = "🫖",
	}
end

local function notFound(_request: net.ServeRequest): net.ServeResponse
	return {
		status = 404,
		body = "Not Found",
	}
end

-- Run the server on port 8080

local handle = net.serve(PORT, function(request)
	if string.sub(request.path, 1, 5) == "/ping" then
		return pong(request)
	elseif string.sub(request.path, 1, 7) == "/teapot" then
		return teapot(request)
	else
		return notFound(request)
	end
end)

print(`Listening on port {PORT} 🚀`)

-- Exit our example after a small delay, if you copy this
-- example just remove this part to keep the server running

task.delay(2, function()
	print("Shutting down...")
	task.wait(1)
	handle:stop()
end)
