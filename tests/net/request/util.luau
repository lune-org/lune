local util = {}

function util.pass(method, url, message)
	local response = net.request({
		method = method,
		url = url,
	})
	if not response.ok then
		error(string.format("%s failed!\nResponse: %s", message, stdio.format(response)))
	end
end

function util.fail(method, url, message)
	local response = net.request({
		method = method,
		url = url,
	})
	if response.ok then
		error(string.format("%s passed!\nResponse: %s", message, stdio.format(response)))
	end
end

return util
