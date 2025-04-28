local net = require("@lune/net")

local handle = net.serve(8080, function()
	return "Hello, World!"
end)

print(`Server started on {handle.ip}:{handle.port}`)
