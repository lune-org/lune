local roblox = require("@lune/roblox") :: any
local NumberSequenceKeypoint = roblox.NumberSequenceKeypoint

-- Constructors & properties

NumberSequenceKeypoint.new(0, 0)
NumberSequenceKeypoint.new(1, 0)
NumberSequenceKeypoint.new(0.5, 0.5, 0.5)

assert(not pcall(function()
	return NumberSequenceKeypoint.new()
end))
assert(not pcall(function()
	return NumberSequenceKeypoint.new(false)
end))
assert(not pcall(function()
	return NumberSequenceKeypoint.new("", "")
end))
assert(not pcall(function()
	return NumberSequenceKeypoint.new(newproxy(true))
end))

assert(NumberSequenceKeypoint.new(0, 0, 0).Time == 0)
assert(NumberSequenceKeypoint.new(1, 0, 1).Time == 1)
assert(NumberSequenceKeypoint.new(0, 0, 0).Value == 0)
assert(NumberSequenceKeypoint.new(1, 1, 1).Value == 1)
assert(NumberSequenceKeypoint.new(0, 0, 0).Envelope == 0)
assert(NumberSequenceKeypoint.new(1, 1, 1).Envelope == 1)
