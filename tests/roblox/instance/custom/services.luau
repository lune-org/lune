local roblox = require("@lune/roblox")

-- Services that rbx-dom does not know about (such as FileSystemService, which
-- only exists in roblox-cli builds) should be implementable (#125)

local game = roblox.Instance.new("DataModel") :: any

local successBefore = pcall(function()
	game:GetService("FileSystemService")
end)
assert(not successBefore, "Getting an unimplemented service should error")

roblox.registerService("FileSystemService")

-- Once implemented, GetService should create and return the service

local fs = game:GetService("FileSystemService")
assert(fs ~= nil, "GetService should return the implemented service")
assert(fs.ClassName == "FileSystemService", "Service should have the correct ClassName")
assert(fs.Parent == game, "Service should be parented to the DataModel")

-- GetService should return the same service instance on subsequent calls

assert(game:GetService("FileSystemService") == fs, "GetService should be idempotent")

-- FindService should locate the existing service

assert(game:FindService("FileSystemService") == fs, "FindService should find the service")

-- Implemented services should be a service, and still inherit from Instance

assert(fs:IsA("FileSystemService"), "Service should be a FileSystemService")
assert(fs:IsA("Instance"), "Service should inherit from Instance")

-- Methods and properties should be implementable for custom services

roblox.implementMethod("FileSystemService", "ReadFile", function(_, path: string)
	return "contents of " .. path
end)
assert(fs:ReadFile("foo.txt") == "contents of foo.txt", "Custom service methods should work")

-- FindService should return nil for a valid-but-not-yet-created service

local game2 = roblox.Instance.new("DataModel") :: any
assert(
	game2:FindService("FileSystemService") == nil,
	"FindService should return nil when the service has not been created"
)
