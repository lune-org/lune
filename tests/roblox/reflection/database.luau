local roblox = require("@lune/roblox")

local db = roblox.getReflectionDatabase()
local db2 = roblox.getReflectionDatabase()

-- Subsequent calls to getReflectionDatabase should return the same database
assert(db == db2, "Database should always compare as equal to other database")

-- Database should not be empty
assert(#db:GetClassNames() > 0, "Database should not be empty (no class names)")
assert(#db:GetEnumNames() > 0, "Database should not be empty (no enum names)")

-- Make sure our database finds classes correctly

local class = db:GetClass("Instance")
assert(class ~= nil, "Missing Instance class in database")
local prop = class.Properties.Parent
assert(prop ~= nil, "Missing Parent property on Instance class in database")

local class2 = db:FindClass("    instance ")
assert(class2 ~= nil, "Missing Instance class in database (2)")
local prop2 = class2.Properties.Parent
assert(prop2 ~= nil, "Missing Parent property on Instance class in database (2)")

assert(class == class2, "Class userdatas from the database should compare as equal")
assert(prop == prop2, "Property userdatas from the database should compare as equal")

assert(db:GetClass("PVInstance") ~= nil, "Missing PVInstance class in database")
assert(db:GetClass("BasePart") ~= nil, "Missing BasePart class in database")
assert(db:GetClass("Part") ~= nil, "Missing Part class in database")

-- Make sure our database finds enums correctly

local enum = db:GetEnum("PartType")
assert(enum ~= nil, "Missing PartType enum in database")

local enum2 = db:FindEnum("   parttype ")
assert(enum2 ~= nil, "Missing PartType enum in database (2)")

assert(enum == enum2, "Enum userdatas from the database should compare as equal")

assert(db:GetEnum("UserInputType") ~= nil, "Missing UserInputType enum in database")
assert(db:GetEnum("NormalId") ~= nil, "Missing NormalId enum in database")
assert(db:GetEnum("Font") ~= nil, "Missing Font enum in database")

-- All the class and enum names gotten from the database should be accessible

for _, className in db:GetClassNames() do
	assert(db:GetClass(className) ~= nil, "Missing " .. className .. " class in database (3)")
	assert(db:FindClass(className) ~= nil, "Missing " .. className .. " class in database (4)")
end
for _, enumName in db:GetEnumNames() do
	assert(db:GetEnum(enumName) ~= nil, "Missing " .. enumName .. " enum in database (3)")
	assert(db:FindEnum(enumName) ~= nil, "Missing " .. enumName .. " enum in database (4)")
end
