local RESPONSE = "Hello, lune!"

task.spawn(function()
	net.serve(8080, function(request)
		-- info("Request:", request)
		-- info("Responding with", RESPONSE)
		return RESPONSE
	end)
end)

local response = net.request("http://127.0.0.1:8080").body
assert(response == RESPONSE, "Invalid response from server")

task.delay(1, function()
	process.exit(0)
end)

task.delay(2, function()
	error("Process did not exit")
end)
