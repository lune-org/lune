local function test(path: string)
	local success, message = pcall(function()
		local _ = require(path) :: any
	end)
	if success then
		error(string.format("Invalid require at path '%s' succeeded", path))
	else
		message = tostring(message)
		if string.find(message, string.format("'%s'", path)) == nil then
			error(
				string.format(
					"Invalid require did not mention path '%s' in its error message!\nMessage: %s",
					path,
					tostring(message)
				)
			)
		end
	end
end

test("foo")
test("bar")
test("moduuuuule")
test("modules.nested")
test(" modules ")
test("mod" .. string.char(127) .. "ules")
