local util = {}

function util.pass(method, url, message: string)
	local response = net.request({
		method = method,
		url = url,
	})
	assert(response.ok, message)
end

function util.fail(method, url, message: string)
	local response = net.request({
		method = method,
		url = url,
	})
	assert(not response.ok, message)
end

return util
