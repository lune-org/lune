--!nocheck
local ffi = require("@lune/ffi")

local function callableWrapper(
	functionRef: ffi.RefData,
	argTypeList: { ffi.CTypes },
	retType: ffi.CTypes
): (...any) -> any
	local callable = ffi.c.fn(argTypeList, retType):callable(functionRef)

	return function(...)
		local argValues = table.create(#argTypeList + 1)

		local resultBox
		if retType ~= ffi.c.void then
			resultBox = ffi.box(retType.size)
		end
		argValues[1] = resultBox

		for index, argType in argTypeList do
			local arg = select(index, ...)
			if type(arg) == "userdata" then
				argValues[index + 1] = arg
			else
				argValues[index + 1] = argType:box(arg):ref()
			end
		end

		callable(table.unpack(argValues, 1, #argTypeList + 1))

		if retType == ffi.c.void then
			return nil
		end
		return retType:readData(resultBox)
	end
end

return callableWrapper
