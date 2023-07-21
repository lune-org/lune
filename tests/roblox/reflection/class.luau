local roblox = require("@lune/roblox")

local db = roblox.getReflectionDatabase()

-- Make sure database classes exist + fields / properties are correct types

for _, className in db:GetClassNames() do
	local class = db:GetClass(className)
	assert(class ~= nil, "Missing " .. className .. " class in database")
	assert(type(class.Name) == "string", "Name property must be a string")
	assert(
		class.Superclass == nil or type(class.Superclass) == "string",
		"Superclass property must be nil or a string"
	)
	assert(type(class.Properties) == "table", "Properties property must be a table")
	assert(type(class.DefaultProperties) == "table", "DefaultProperties property must be a table")
	assert(type(class.Tags) == "table", "Tags property must be a table")
end

-- Any property present in default properties must also
-- be in properties *or* the properties of a superclass

for _, className in db:GetClassNames() do
	local class = db:GetClass(className)
	assert(class ~= nil)
	for name, value in class.DefaultProperties do
		local found = false
		local current = class
		while current ~= nil do
			if current.Properties[name] ~= nil then
				found = true
				break
			elseif current.Superclass ~= nil then
				current = db:GetClass(current.Superclass)
			else
				break
			end
		end
		assert(found, "Missing default property " .. name .. " in properties table")
	end
end
