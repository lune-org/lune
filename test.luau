local net = require("@lune/net")

local counter = 0
net.serve(8080, function()
	counter += 1
	return {
		status = 200,
		body = "Hello! This is response #" .. tostring(counter),
	}
end)

print("Listening on port 8080 🚀")
