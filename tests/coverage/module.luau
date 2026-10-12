local module = {}

function module.double(value: number): number
	return value * 2
end

function module.unused(value: number): number
	return value
end

return module
