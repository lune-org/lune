--> A basic http server that echoes the given request
--> body at /ping and otherwise responds 404 "Not Found"

local net = require("@lune/net")
local process = require("@lune/process")

local PORT = if process.env.PORT ~= nil and #process.env.PORT > 0
	then assert(tonumber(process.env.PORT), "Failed to parse port from env")
	else 8080

-- Create our responder functions

local function root(_request: net.ServeRequest): string
	return `Hello from Lune server!`
end

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

-- Run the server on the port forever

net.serve(PORT, function(request)
	if request.path == "/" then
		return root(request)
	elseif string.sub(request.path, 1, 5) == "/ping" then
		return pong(request)
	elseif string.sub(request.path, 1, 7) == "/teapot" then
		return teapot(request)
	else
		return notFound(request)
	end
end)

print(`Listening on port {PORT} 🚀`)
print("Press Ctrl+C to stop")
