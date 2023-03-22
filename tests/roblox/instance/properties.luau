local roblox = require("@lune/roblox") :: any
local BrickColor = roblox.BrickColor
local Instance = roblox.Instance
local Vector3 = roblox.Vector3
local CFrame = roblox.CFrame
local Enum = roblox.Enum

local part = Instance.new("Part")

-- Primitive type properties should work (note that these are inherited from BasePart)

part.Anchored = true
part.CanCollide = true
part.CanQuery = false

assert(part.Anchored == true)
assert(part.CanCollide == true)
assert(part.CanQuery == false)

-- More complex types like Vector3 should work

part.Size = Vector3.one
part.CFrame = CFrame.identity
part.BrickColor = BrickColor.Red()

assert(part.Size == Vector3.one)
assert(part.CFrame == CFrame.identity)
assert(part.BrickColor == BrickColor.Red())

-- Enums should work (note that these are specific to Part and not on BasePart)

part.Shape = Enum.PartType.Ball

assert(part.Shape == Enum.PartType.Ball)

-- Properties that don't exist for a class should error

local meshPart = Instance.new("MeshPart")

assert(not pcall(function()
	meshPart.Shape = Enum.PartType.Ball
end))

-- We should be able to access properties without first setting them

assert(meshPart.Anchored == false)
assert(meshPart.Material == Enum.Material.Plastic)
assert(meshPart.Size == Vector3.new(4, 1.2, 2))
assert(meshPart.CustomPhysicalProperties == nil)
