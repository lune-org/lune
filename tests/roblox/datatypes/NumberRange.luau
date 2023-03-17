-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local NumberRange = _G.NumberRange

-- Constructors & properties

NumberRange.new(0)
NumberRange.new(0, 0)

assert(not pcall(function()
	return NumberRange.new()
end))
assert(not pcall(function()
	return NumberRange.new(false)
end))
assert(not pcall(function()
	return NumberRange.new("", "")
end))
assert(not pcall(function()
	return NumberRange.new(newproxy(true))
end))

assert(NumberRange.new(0, 1).Min == 0)
assert(NumberRange.new(1, 1).Min == 1)

assert(NumberRange.new(0, 0).Max == 0)
assert(NumberRange.new(0, 1).Max == 1)

-- Swapped args should still set proper min/max

assert(NumberRange.new(1, 0).Min == 0)
assert(NumberRange.new(1, 0).Max == 1)
