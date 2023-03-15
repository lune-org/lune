-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local Color3 = _G.Color3
local ColorSequence = _G.ColorSequence
local ColorSequenceKeypoint = _G.ColorSequenceKeypoint

-- Constructors & properties

ColorSequence.new(Color3.new())
ColorSequence.new(Color3.new(), Color3.new())
local sequence = ColorSequence.new({
	ColorSequenceKeypoint.new(0, Color3.new(1, 0, 0)),
	ColorSequenceKeypoint.new(0.5, Color3.new(0, 1, 0)),
	ColorSequenceKeypoint.new(1, Color3.new(0, 0, 1)),
})

assert(not pcall(function()
	return ColorSequence.new()
end))
assert(not pcall(function()
	return ColorSequence.new(0.5)
end))
assert(not pcall(function()
	return ColorSequence.new(false)
end))
assert(not pcall(function()
	return ColorSequence.new("", "")
end))
assert(not pcall(function()
	return ColorSequence.new(newproxy(true))
end))

assert(#sequence.Keypoints == 3)
assert(sequence.Keypoints[1] == ColorSequenceKeypoint.new(0, Color3.new(1, 0, 0)))
assert(sequence.Keypoints[2] == ColorSequenceKeypoint.new(0.5, Color3.new(0, 1, 0)))
assert(sequence.Keypoints[3] == ColorSequenceKeypoint.new(1, Color3.new(0, 0, 1)))
