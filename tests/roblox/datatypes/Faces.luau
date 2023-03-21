local roblox = require("@lune/roblox") :: any
local Faces = roblox.Faces
local Enum = roblox.Enum

-- Constructors & properties

Faces.new()
Faces.new(Enum.NormalId.Top)
Faces.new(Enum.NormalId.Left, Enum.NormalId.Top)

assert(not pcall(function()
	return Faces.new(false)
end))
assert(not pcall(function()
	return Faces.new({})
end))
assert(not pcall(function()
	return Faces.new(newproxy(true))
end))

assert(Faces.new().Left == false)
assert(Faces.new().Right == false)
assert(Faces.new().Top == false)
assert(Faces.new().Bottom == false)
assert(Faces.new().Front == false)
assert(Faces.new().Back == false)

assert(Faces.new(Enum.NormalId.Left).Left == true)
assert(Faces.new(Enum.NormalId.Right).Right == true)

local f = Faces.new(
	Enum.NormalId.Left,
	Enum.NormalId.Right,
	Enum.NormalId.Top,
	Enum.NormalId.Bottom,
	Enum.NormalId.Back
)
assert(f.Left == true)
assert(f.Right == true)
assert(f.Top == true)
assert(f.Bottom == true)
assert(f.Front == false)
assert(f.Back == true)

-- Ops

assert(not pcall(function()
	return Faces.new() + Faces.new()
end))
assert(not pcall(function()
	return Faces.new() / Faces.new()
end))
