local roblox = require("@lune/roblox") :: any
local CFrame = roblox.CFrame
local Vector3 = roblox.Vector3

-- Constructors & properties

CFrame.new()
CFrame.new(0, 0, 0)
CFrame.new(0 / 0, 0 / 0, 0 / 0)
CFrame.new(0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1)

assert(not pcall(function()
	return CFrame.new(false)
end))
assert(not pcall(function()
	return CFrame.new("", "")
end))
assert(not pcall(function()
	return CFrame.new(newproxy(true))
end))

assert(CFrame.new(1, 2, 3).X == 1)
assert(CFrame.new(1, 2, 3).Y == 2)
assert(CFrame.new(1, 2, 3).Z == 3)

-- Constants

assert(CFrame.identity == CFrame.new())
assert(CFrame.identity == CFrame.new(0, 0, 0))
assert(CFrame.identity == CFrame.Angles(0, 0, 0))
assert(CFrame.identity == CFrame.fromOrientation(0, 0, 0))

-- Ops

assert(CFrame.new(2, 4, 8) + Vector3.new(1, 1, 2) == CFrame.new(3, 5, 10))
assert(CFrame.new(2, 4, 8) - Vector3.new(1, 1, 2) == CFrame.new(1, 3, 6))
assert(CFrame.new(2, 4, 8) * CFrame.new(1, 1, 2) == CFrame.new(3, 5, 10))
assert(CFrame.new(2, 4, 8) * Vector3.new(1, 1, 2) == Vector3.new(3, 5, 10))

-- TODO: Check mult ops with rotated CFrames

-- TODO: Methods
