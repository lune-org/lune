--> A basic http server that echoes the given request
--> body at /ping and otherwise responds 404 "Not Found"

local PORT = if process.env.PORT ~= nil and #process.env.PORT > 0
	then assert(tonumber(process.env.PORT), "Failed to parse port from env")
	else 8080

-- Create our responder functions

local function pong(request: NetRequest): string
	return `Pong!\n{request.path}\n{request.body}`
end

local function teapot(request: NetRequest): NetResponse
	return {
		status = 418,
		body = "🫖",
	}
end

local function notFound(request: NetRequest): NetResponse
	return {
		status = 404,
		body = "Not Found",
	}
end

-- Exit our example after a small delay, if you copy this
-- example just remove this part to keep the server running

task.delay(2, function()
	print("Shutting down...")
	task.wait(1)
	process.exit(0)
end)

-- Run the server on port 8080

print(`Listening on port {PORT} 🚀`)
net.serve(PORT, function(request)
	if string.sub(request.path, 1, 5) == "/ping" then
		return pong(request)
	elseif string.sub(request.path, 1, 7) == "/teapot" then
		return teapot(request)
	else
		return notFound(request)
	end
end)
