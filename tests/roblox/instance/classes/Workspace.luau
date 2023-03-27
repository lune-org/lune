local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance

local game = Instance.new("DataModel")
local workspace = game:GetService("Workspace")

-- Terrain should always exist as a "Terrain" property, or be created when accessed

assert(workspace.Terrain ~= nil)
assert(workspace.Terrain:IsA("Terrain"))
assert(workspace.Terrain == workspace:FindFirstChildOfClass("Terrain"))

-- Camera should always exist as a "CurrentCamera" property, or be created when accessed

assert(workspace.CurrentCamera ~= nil)
assert(workspace.CurrentCamera:IsA("Camera"))
assert(workspace.CurrentCamera == workspace:FindFirstChildOfClass("Camera"))
