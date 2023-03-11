-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local UDim = _G.UDim

-- Constructors & properties

UDim.new()
UDim.new(0)
UDim.new(0, 0)
UDim.new(0 / 0, 0)

assert(not pcall(function()
	return UDim.new(false)
end))
assert(not pcall(function()
	return UDim.new("", "")
end))
assert(not pcall(function()
	return UDim.new(newproxy(true))
end))

assert(UDim.new(1, 2).Scale == 1)
assert(UDim.new(1, 2).Offset == 2)

-- Ops

assert(UDim.new(2, 4) + UDim.new(1, 1) == UDim.new(3, 5))
assert(UDim.new(2, 4) - UDim.new(1, 1) == UDim.new(1, 3))
