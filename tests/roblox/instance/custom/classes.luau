local roblox = require("@lune/roblox")

-- Custom classes that rbx-dom does not know about should be implementable (#125)

local successBefore = pcall(function()
	roblox.Instance.new("MyCustomClass")
end)
assert(not successBefore, "Creating an unimplemented custom class should error")

roblox.registerClass("MyCustomClass")

-- Once implemented, creating instances of it should work

local custom = roblox.Instance.new("MyCustomClass") :: any
assert(custom.ClassName == "MyCustomClass", "ClassName should reflect the custom class")

-- Custom classes should inherit from Instance by default

assert(custom:IsA("MyCustomClass"), "Custom instance should be a MyCustomClass")
assert(custom:IsA("Instance"), "Custom instance should inherit from Instance")
assert(not custom:IsA("Part"), "Custom instance should not be an unrelated class")

-- Standard instance members should still work on custom classes

custom.Name = "Hello"
assert(custom.Name == "Hello", "Setting Name on a custom instance should work")

local child = roblox.Instance.new("Folder")
child.Parent = custom
assert(child.Parent == custom, "Custom instances should support children")
assert(custom:FindFirstChild("Folder") == child, "FindFirstChild should work on custom instances")

-- Methods and properties should be implementable for custom classes

roblox.implementMethod("MyCustomClass", "DoThing", function(instance, value)
	return instance, value
end)
local returnedInstance, returnedValue = custom:DoThing(42)
assert(returnedInstance == custom, "Custom method should receive the instance")
assert(returnedValue == 42, "Custom method should receive and return arguments")

roblox.implementProperty("MyCustomClass", "Magic", function()
	return 123
end)
assert(custom.Magic == 123, "Custom property getter should work on custom classes")

-- Implementing a class that already exists should error

local successExisting = pcall(function()
	roblox.registerClass("Part")
end)
assert(not successExisting, "Implementing an existing class should error")

local successDuplicate = pcall(function()
	roblox.registerClass("MyCustomClass")
end)
assert(not successDuplicate, "Implementing the same custom class twice should error")

-- Implementing a class with an invalid superclass should error

local successBadSuper = pcall(function()
	roblox.registerClass("AnotherClass", "NotARealClass")
end)
assert(not successBadSuper, "Implementing a class with an invalid superclass should error")

-- Custom classes should be able to inherit from other custom classes,
-- including inheriting their implemented methods

roblox.registerClass("CustomBase")
roblox.implementMethod("CustomBase", "BaseMethod", function()
	return "from base"
end)
roblox.registerClass("CustomDerived", "CustomBase")

local derived = roblox.Instance.new("CustomDerived") :: any
assert(derived:IsA("CustomBase"), "Derived custom class should inherit from its custom superclass")
assert(derived:IsA("Instance"), "Derived custom class should still inherit from Instance")
assert(derived:BaseMethod() == "from base", "Inherited custom methods should work")

-- Custom (non-service) classes should not be retrievable via GetService

local game = roblox.Instance.new("DataModel") :: any
local successGetService = pcall(function()
	game:GetService("MyCustomClass")
end)
assert(not successGetService, "Non-service custom classes should not be valid services")
