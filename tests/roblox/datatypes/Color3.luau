local roblox = require("@lune/roblox") :: any
local Color3 = roblox.Color3

-- Constructors & properties

Color3.new()
Color3.new(0)
Color3.new(0, 0)
Color3.new(0, 0, 0)
Color3.new(0 / 0, 0 / 0)
Color3.new(0 / 0, 0 / 0, 0 / 0)

assert(not pcall(function()
	return Color3.new(false)
end))
assert(not pcall(function()
	return Color3.new("", "")
end))
assert(not pcall(function()
	return Color3.new(newproxy(true))
end))

assert(Color3.new(0.25, 0.5, 1).R == 0.25)
assert(Color3.new(0.25, 0.5, 1).G == 0.5)
assert(Color3.new(0.25, 0.5, 1).B == 1)

assert(Color3.fromRGB(0, 0, 0) == Color3.new(0, 0, 0))
assert(Color3.fromRGB(255, 255, 255) == Color3.new(1, 1, 1))

assert(Color3.fromHex("00F") == Color3.new(0, 0, 1))
assert(Color3.fromHex("0000FF") == Color3.new(0, 0, 1))

assert(Color3.fromHSV(0, 1, 1) == Color3.new(1, 0, 0))
assert(Color3.fromHSV(0, 1, 0) == Color3.new(0, 0, 0))
assert(Color3.fromHSV(0, 0, 1) == Color3.new(1, 1, 1))

-- Ops

assert(Color3.new(0.25, 0.5, 1) + Color3.new(0.25, 0.25, 0.25) == Color3.new(0.5, 0.75, 1.25))
assert(Color3.new(0.25, 0.5, 1) - Color3.new(0.25, 0.25, 0.25) == Color3.new(0, 0.25, 0.75))
assert(Color3.new(0.25, 0.5, 1) * Color3.new(0.25, 0.25, 0.5) == Color3.new(0.0625, 0.125, 0.5))
assert(Color3.new(0.25, 0.5, 1) / Color3.new(0.25, 0.25, 0.5) == Color3.new(1, 2, 2))

assert(Color3.new(0.25, 0.5, 1) * 2 == Color3.new(0.5, 1, 2))
assert(Color3.new(0.25, 0.5, 1) / 2 == Color3.new(0.125, 0.25, 0.5))

-- Methods

local h, s, v

h, s, v = Color3.fromHSV(0, 0.25, 0.75):ToHSV()
assert(h == 0 and s == 0.25 and v == 0.75)

h, s, v = Color3.fromHSV(0.5, 1, 0.125):ToHSV()
assert(h == 0.5 and s == 1 and v == 0.125)

assert(Color3.fromHex("FFF"):ToHex() == "FFFFFF")
assert(Color3.fromHex("FA0"):ToHex() == "FFAA00")
assert(Color3.fromHex("FFFFFF"):ToHex() == "FFFFFF")
assert(Color3.fromHex("FFAA00"):ToHex() == "FFAA00")
