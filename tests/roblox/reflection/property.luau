local roblox = require("@lune/roblox")

local db = roblox.getReflectionDatabase()

-- Make sure database class properties exist + their fields / properties are correct types

for _, className in db:GetClassNames() do
	local class = db:GetClass(className)
	assert(class ~= nil)

	for name, prop in class.Properties do
		assert(type(prop.Name) == "string", "Name property must be a string")
		assert(type(prop.Datatype) == "string", "Datatype property must be a string")
		assert(type(prop.Scriptability) == "string", "Scriptability property must be a string")
		assert(type(prop.Tags) == "table", "Tags property must be a table")
	end
end
