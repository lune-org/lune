local roblox = require("@lune/roblox") :: any
local TweenInfo = roblox.TweenInfo
local Enum = roblox.Enum

-- Constructors & properties

TweenInfo.new()
TweenInfo.new(1)
TweenInfo.new(1, Enum.EasingStyle.Sine)
TweenInfo.new(1, Enum.EasingStyle.Sine, Enum.EasingDirection.Out)
TweenInfo.new(1, Enum.EasingStyle.Sine, Enum.EasingDirection.Out, 2, true, 0.5)

-- Defaults should match Roblox

local default = TweenInfo.new()
assert(default.Time == 1)
assert(default.EasingStyle == Enum.EasingStyle.Quad)
assert(default.EasingDirection == Enum.EasingDirection.Out)
assert(default.RepeatCount == 0)
assert(default.Reverses == false)
assert(default.DelayTime == 0)

-- All properties should round-trip

local info = TweenInfo.new(2.5, Enum.EasingStyle.Bounce, Enum.EasingDirection.InOut, 3, true, 0.25)
assert(info.Time == 2.5)
assert(info.EasingStyle == Enum.EasingStyle.Bounce)
assert(info.EasingDirection == Enum.EasingDirection.InOut)
assert(info.RepeatCount == 3)
assert(info.Reverses == true)
assert(info.DelayTime == 0.25)

-- Wrong enum types should error

assert(not pcall(function()
	return TweenInfo.new(1, Enum.EasingDirection.Out)
end))
assert(not pcall(function()
	return TweenInfo.new(1, Enum.EasingStyle.Sine, Enum.EasingStyle.Sine)
end))
assert(not pcall(function()
	return TweenInfo.new(1, Enum.KeyCode.A)
end))
assert(not pcall(function()
	return TweenInfo.new(false)
end))

-- Equality

assert(TweenInfo.new(1, Enum.EasingStyle.Sine) == TweenInfo.new(1, Enum.EasingStyle.Sine))
assert(TweenInfo.new(1) ~= TweenInfo.new(2))
assert(TweenInfo.new(1, Enum.EasingStyle.Sine) ~= TweenInfo.new(1, Enum.EasingStyle.Linear))
assert(TweenInfo.new(1, Enum.EasingStyle.Sine, Enum.EasingDirection.In) ~= TweenInfo.new(1, Enum.EasingStyle.Sine, Enum.EasingDirection.Out))

-- tostring

assert(type(tostring(TweenInfo.new())) == "string")
