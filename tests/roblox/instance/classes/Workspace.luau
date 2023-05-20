local roblox = require("@lune/roblox")
local Instance = roblox.Instance

local game = Instance.new("DataModel")
local workspace = game:GetService("Workspace")

-- Terrain should always exist as a "Terrain" property, or be created when accessed

local terrain = (workspace :: any).Terrain
assert(terrain ~= nil)
assert(terrain:IsA("Terrain"))
assert(terrain == workspace:FindFirstChildOfClass("Terrain"))

-- Camera should always exist as a "CurrentCamera" property, or be created when accessed

local camera = (workspace :: any).CurrentCamera
assert(camera ~= nil)
assert(camera:IsA("Camera"))
assert(camera == workspace:FindFirstChildOfClass("Camera"))
