local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local game = Instance.new("DataModel")

-- Workspace should always exist as a "Workspace" property, or be created when accessed

local workspace = (game :: any).Workspace
assert(workspace ~= nil)
assert(workspace:IsA("Workspace"))
assert(workspace == game:FindFirstChildOfClass("Workspace"))

-- GetService and FindService should work, GetService should create services that don't exist

assert(game:FindService("CSGDictionaryService") == nil)
assert(game:GetService("CSGDictionaryService"))
assert(game:FindService("CSGDictionaryService") ~= nil)

-- Service names should be strict and not allow weird characters or substrings

assert(not pcall(function()
	game:GetService("wrorokspacey")
end))

assert(not pcall(function()
	game:GetService("Work-space")
end))

assert(not pcall(function()
	game:GetService("workspac")
end))
