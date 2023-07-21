local roblox = require("@lune/roblox")

local db = roblox.getReflectionDatabase()

-- Make sure database enums exist + fields / properties are correct types

for _, enumName in db:GetEnumNames() do
	local enum = db:GetEnum(enumName)
	assert(enum ~= nil, "Missing " .. enumName .. " enum in database")
	assert(type(enum.Name) == "string", "Name property must be a string")
	assert(type(enum.Items) == "table", "Items property must be a table")
end

-- Enum items should be a non-empty map of string -> positive integer values

for _, enumName in db:GetEnumNames() do
	local enum = db:GetEnum(enumName)
	assert(enum ~= nil)
	local empty = true
	for name, value in enum.Items do
		assert(
			type(name) == "string" and #name > 0,
			"Enum items map must only contain non-empty string keys"
		)
		assert(
			type(value) == "number" and value >= 0 and math.floor(value) == value,
			"Enum items map must only contain positive integer values"
		)
		empty = false
	end
	assert(not empty, "Enum items map must not be empty")
end
