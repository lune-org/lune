local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local game = Instance.new("DataModel")

assert(game:FindService("Workspace") == nil)
assert(game:GetService("Workspace") ~= nil)
assert(game:FindService("Workspace") ~= nil)

assert(game:FindService("CSGDictionaryService") == nil)
assert(game:GetService("CSGDictionaryService") ~= nil)
assert(game:FindService("CSGDictionaryService") ~= nil)

assert(not pcall(function()
	game:GetService("wrorokspacey")
end))

assert(not pcall(function()
	game:GetService("work-space")
end))

assert(not pcall(function()
	game:GetService("workspac")
end))
