local PORT = 9090 -- NOTE: This must be different from
-- net tests to let them run in parallel with this file

local function test(f, ...)
	local success, message = pcall(f, ...)
	assert(not success, "Function did not throw an error")
	assert(type(message) == "userdata", "Pcall did not return a proper error")
end

-- These are not async but should be pcallable

test(error, "Test error", 2)

-- Net request is async and will throw a DNS error here for the weird address

test(net.request, "https://wxyz.google.com")

-- Net serve is async and will throw an OS error when trying to serve twice on the same port

local handle = net.serve(PORT, function()
	return ""
end)

task.delay(0, function()
	handle.stop()
end)

test(net.serve, PORT, function() end)
