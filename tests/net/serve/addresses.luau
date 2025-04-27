local net = require("@lune/net")

local PORT = 8081
local LOCALHOST = "http://localhost"
local BROADCAST = `http://0.0.0.0`
local RESPONSE = "Hello, lune!"

-- Serve should be able to bind to broadcast IP addresse

local handle = net.serve(PORT, {
	address = BROADCAST,
	handleRequest = function(request)
		return `Response from {BROADCAST}:{PORT}`
	end,
})

-- And any requests to localhost should then succeed

local response = net.request(`{LOCALHOST}:{PORT}`).body
assert(response ~= nil, "Invalid response from server")

handle:stop()

-- Attempting to serve with a malformed IP address should throw an error

local success = pcall(function()
	net.serve(8080, {
		address = "a.b.c.d",
		handleRequest = function()
			return RESPONSE
		end,
	})
end)

assert(not success, "Server was created with malformed address")
