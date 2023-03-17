-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local NumberSequence = _G.NumberSequence
local NumberSequenceKeypoint = _G.NumberSequenceKeypoint

-- Constructors & properties

NumberSequence.new(0)
NumberSequence.new(0, 0)
local sequence = NumberSequence.new({
	NumberSequenceKeypoint.new(0, 1),
	NumberSequenceKeypoint.new(0.5, 0.5),
	NumberSequenceKeypoint.new(1, 0),
})

assert(not pcall(function()
	return NumberSequence.new()
end))
assert(not pcall(function()
	return NumberSequence.new(false)
end))
assert(not pcall(function()
	return NumberSequence.new("", "")
end))
assert(not pcall(function()
	return NumberSequence.new(newproxy(true))
end))

assert(#sequence.Keypoints == 3)
assert(sequence.Keypoints[1] == NumberSequenceKeypoint.new(0, 1))
assert(sequence.Keypoints[2] == NumberSequenceKeypoint.new(0.5, 0.5))
assert(sequence.Keypoints[3] == NumberSequenceKeypoint.new(1, 0))
