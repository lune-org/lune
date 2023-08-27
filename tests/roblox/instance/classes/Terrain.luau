local roblox = require("@lune/roblox") :: any
local Instance = roblox.Instance
local Color3 = roblox.Color3
local Enum = roblox.Enum

local game = Instance.new("DataModel")
local workspace = game:GetService("Workspace")
local terrain = (workspace :: any).Terrain

assert(terrain:GetMaterialColor(Enum.Material.Grass) == Color3.fromRGB(106, 127, 63))

terrain:SetMaterialColor(Enum.Material.Sand, Color3.new(1, 1, 1))
assert(terrain:GetMaterialColor(Enum.Material.Sand) == Color3.new(1, 1, 1))
