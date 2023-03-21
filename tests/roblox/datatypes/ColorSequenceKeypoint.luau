local roblox = require("@lune/roblox") :: any
local Color3 = roblox.Color3
local ColorSequenceKeypoint = roblox.ColorSequenceKeypoint

-- Constructors & properties

ColorSequenceKeypoint.new(0, Color3.new())
ColorSequenceKeypoint.new(1, Color3.new())

assert(not pcall(function()
	return ColorSequenceKeypoint.new()
end))
assert(not pcall(function()
	return ColorSequenceKeypoint.new(0.5)
end))
assert(not pcall(function()
	return ColorSequenceKeypoint.new(false)
end))
assert(not pcall(function()
	return ColorSequenceKeypoint.new("", "")
end))
assert(not pcall(function()
	return ColorSequenceKeypoint.new(newproxy(true))
end))

assert(ColorSequenceKeypoint.new(0, Color3.new()).Time == 0)
assert(ColorSequenceKeypoint.new(1, Color3.new()).Time == 1)
assert(ColorSequenceKeypoint.new(0, Color3.new()).Value == Color3.new())
assert(ColorSequenceKeypoint.new(1, Color3.new(1)).Value == Color3.new(1))
