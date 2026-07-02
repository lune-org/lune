local net = require("@lune/net")
local stdio = require("@lune/stdio")

local util = {}

function util.pass(method, url, message)
	local success, response = pcall(net.request, {
		method = method,
		url = url,
	})
	if not success then
		error(`{message} errored!\nError message: {tostring(response)}`)
	elseif not response.ok then
		error(
			`{message} failed, but should have passed!`
				.. `\nStatus code: {response.statusCode}`
				.. `\nStatus message: {response.statusMessage}`
				.. `\nResponse headers: {stdio.format(response.headers)}`
				.. `\nResponse body: {response.body}`
		)
	end
end

function util.fail(method, url, message)
	local success, response = pcall(net.request, {
		method = method,
		url = url,
	})
	if not success then
		error(`{message} errored!\nError message: {tostring(response)}`)
	elseif response.ok then
		error(
			`{message} passed, but should have failed!`
				.. `\nStatus code: {response.statusCode}`
				.. `\nStatus message: {response.statusMessage}`
				.. `\nResponse headers: {stdio.format(response.headers)}`
				.. `\nResponse body: {response.body}`
		)
	end
end

return util
