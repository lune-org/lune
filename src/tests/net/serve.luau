local RESPONSE = "Hello, lune!"

task.spawn(function()
	net.serve(8080, function(request)
		console.info("Request:", request)
		console.info("Responding with", RESPONSE)
		return RESPONSE
	end)
end)

local response = net.request("http://127.0.0.1:8080").body
assert(response == RESPONSE, "Invalid response from server")

task.delay(1, function()
	process.exit(0)
end)

task.delay(2, function()
	console.error("Process did not exit")
	process.exit(1)
end)
