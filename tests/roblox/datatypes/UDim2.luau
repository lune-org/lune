-- HACK: Make luau happy, with the mlua rust
-- crate all globals are also present in _G
local UDim = _G.UDim
local UDim2 = _G.UDim2

-- Constructors & properties

UDim2.new()
UDim2.new(0)
UDim2.new(0, 0)
UDim2.new(0, 0, 0)
UDim2.new(0, 0, 0, 0)
UDim2.new(0 / 0, 0, 0 / 0, 0)

assert(not pcall(function()
	return UDim2.new(false)
end))
assert(not pcall(function()
	return UDim2.new("", "")
end))
assert(not pcall(function()
	return UDim2.new(newproxy(true))
end))

UDim2.fromScale()
UDim2.fromScale(0)
UDim2.fromScale(0, 0)

UDim2.fromOffset()
UDim2.fromOffset(0)
UDim2.fromOffset(0, 0)

assert(UDim2.fromScale(1, 1).X == UDim.new(1, 0))
assert(UDim2.fromScale(1, 1).Y == UDim.new(1, 0))
assert(UDim2.fromScale(1, 1).Width == UDim.new(1, 0))
assert(UDim2.fromScale(1, 1).Height == UDim.new(1, 0))

assert(UDim2.fromOffset(1, 1).X == UDim.new(0, 1))
assert(UDim2.fromOffset(1, 1).Y == UDim.new(0, 1))
assert(UDim2.fromOffset(1, 1).Width == UDim.new(0, 1))
assert(UDim2.fromOffset(1, 1).Height == UDim.new(0, 1))

-- Ops

assert(UDim2.new(2, 4, 6, 8) + UDim2.new(1, 1, 1, 1) == UDim2.new(3, 5, 7, 9))
assert(UDim2.new(2, 4, 6, 8) - UDim2.new(1, 1, 1, 1) == UDim2.new(1, 3, 5, 7))

-- Methods

assert(UDim2.new(2, 4, 6, 8):Lerp(UDim2.new(1, 2, 3, 4), 0.0) == UDim2.new(2, 3, 6, 8))
assert(UDim2.new(2, 4, 6, 8):Lerp(UDim2.new(1, 2, 3, 4), 0.5) == UDim2.new(1.5, 3, 4.5, 6))
assert(UDim2.new(2, 4, 6, 8):Lerp(UDim2.new(1, 2, 3, 4), 1.0) == UDim2.new(1, 2, 3, 4))
